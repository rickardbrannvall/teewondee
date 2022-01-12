#![allow(non_snake_case)]
use concrete::*;

use std::env;
use std::time::{Instant}; // Duration, 

use std::fs::OpenOptions;
use std::fs::File;
use std::io::prelude::*;

use std::collections::HashMap;

pub struct RollingContext {
    bsk: LWEBSK,
    sk: LWESecretKey,
    xv: VectorLWE, 
    cache: HashMap<Vec<usize>,(f64, VectorLWE)>,
    size: i32,
    boot: i32,
    tag: String,
    file: File,
}

impl RollingContext {

    // this implementation tests calculating rolling averages for window size is N=2^k 
    // it exploites the structure of the binary tree by caching intermediate results
    // it monitors padding of LWE ciphertexts such that bootstrap is deferred until necessary
    // bootstrap to restore padding and rescale is carried out padding reaches 1
    // note that bootstrap is optional at top level (as it may be effecient to defer)
    
    pub fn partial_mean_recursion(&mut self, iv: &[usize], l: i32, enc: &Encoder) -> (f64, VectorLWE) {
        // iv is list of indices for which to find average
        // l: is level parameter for l>0 rescales all levels, otherwise manual rescale of final 
        // enc: is target encoding (from original VectorLWE ciphertext)
        // 
        let n = iv.len();
        assert!(n>0);
        if self.cache.contains_key(iv) {
            //println!("Found {:?} in cache!",iv);
            let (s, rs) = self.cache.get(iv).unwrap();
            return (*s, rs.clone()); 
        }
        
        let m = n/2;
        if n == 1 {
            let idx = iv[0];
            let val = self.xv.extract_nth(idx).unwrap();
            //println!("n, m, p: {}, {} {}", n, m, val.encoders[0].nb_bit_padding);
            return (1.0, val.clone());
        }
        assert!(n-m==m);
        let (ss, rr) = self.partial_mean_recursion(&iv[..m], l+1, &enc);
        let (_s, _r) = self.partial_mean_recursion(&iv[m..], l+1, &enc);
        assert!(rr.encoders[0].nb_bit_padding==_r.encoders[0].nb_bit_padding);
        let mut s = 0.5*ss;
        let mut rs = rr.add_with_padding(&_r).unwrap();
        let p = rs.encoders[0].nb_bit_padding;
        if p == 1 && l > 0 { // bootstrap partial mean
            //println!("n, m, p, s: {}, {}, {}, {} - bootstrap to restore padding and scale", n, m, p, s);
            rs = rs.bootstrap_nth_with_function(&self.bsk, |x| s*x, &enc, 0).unwrap();
            s = 1.0; 
            self.boot += 1;
        }
        self.cache.insert(iv.to_vec(), (s, rs.clone()));
        self.size += 1;
        return (s, rs);
    }
    
    pub fn rolling_mean_of_2toN(&mut self, w: usize, l: i32) -> (f64, VectorLWE) {
        // w: size of rolling window which must be 2^N
        // l: is level parameter for l>0 rescales all levels, otherwise manual rescale of final 
        //
        let n = self.xv.nb_ciphertexts;
        //let n = 2*w; // XXX while testing
        let iv: Vec<usize> = (0..n).collect();
        let k = n - w + 1;
        let mut rv = VectorLWE::zero(self.xv.dimension, k).unwrap(); 
        let base = w/2;
        let mut scale = 1.0;
        let enc = self.xv.encoders[0].clone();

        for i in 0..k {
            let now = Instant::now();
            let m = i/base;
            let c = i - m*base;
            let mut v = (i..(i+w)).map(|i| iv[i]).collect::<Vec<_>>();
            v.rotate_right(c);
            let (s, r) = self.partial_mean_recursion(&v, l, &enc);
            rv.copy_in_nth_nth_inplace(i, &r, 0).unwrap();
            let step_time = now.elapsed().as_millis();
            scale = s;
            let msg = format!("{} {} {} {} {}", self.tag, i, step_time, self.size, self.boot);
            println!("{}", msg);
            writeln!(self.file, "{}", msg).unwrap();    
        }
        return (scale, rv);
    }
}


fn main() -> Result<(), CryptoAPIError> { //calc_rolling keys data window

    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        println!("format: calc_hourly keys data widow");
        println!("example calc_hourly 80_1024_1_6_4 CGM_p77_24h_2_3_400 8");
        return Ok(());
    }
    
    println!("\nsetting parameters ..."); 
    let mut keys = "80_1024_1_6_4";
    let mut data = "CGM_p77_24h_2_3_400";
    let mut window: usize = 8;
    if args.len() == 4 {
        keys = &args[1];
        data = &args[2];
        window =  args[3].parse().unwrap();
    }
    println!("assume: calc_rolling {} {} {}", keys, data, window);

    let sk0_LWE_path = format!("keys/{}/sk0_LWE.json",keys);
    let bsk00_path = format!("keys/{}/bsk00_LWE.json",keys);
    let datafile = format!("data/{}/{}.enc",keys,data);
    let savefile = format!("data/{}/{}_rolling_{}.enc",keys,data,window);
    let log_file = format!("rolling_stats.txt");
    
    
    println!("\nloading keys and data ... ");
    println!("keys: {}",keys);
    println!("data: {}",data);
    println!("logs: {}",log_file);
    let now = Instant::now();
    let mut context = RollingContext{
        bsk: LWEBSK::load(&bsk00_path),
        sk: LWESecretKey::load(&sk0_LWE_path).unwrap(),
        xv: VectorLWE::load(&datafile).unwrap(),
        cache: HashMap::new(),
        size: 0,
        boot: 0,
        tag: format!("{} {} {}", keys, data, window),
        file: OpenOptions::new().append(true).open(log_file).unwrap(),
    };    
    let load_time = now.elapsed().as_millis();
    println!("data dim {} and enc {:?}", context.xv.dimension, context.xv.encoders[0]);
    println!("load keys and data (size {}) took {} ms\n", context.xv.nb_ciphertexts, load_time);
    
    println!("\ncalculate rolling average ... ");
    let w = 8;
    let (_s, avg) = context.rolling_mean_of_2toN(w,1);
    println!("avg: {:?}", avg.decrypt_decode(&context.sk).unwrap());
    
    println!("\nsave result in file {}", savefile);
    avg.save(&savefile).unwrap();    
       
    Ok(())
}
