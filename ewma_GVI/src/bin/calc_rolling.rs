#![allow(non_snake_case)]
use concrete::*;

use std::env;
use std::time::{Instant}; // Duration, 

//use std::fs::OpenOptions;
//use std::io::prelude::*;

use std::collections::HashMap;

//pub struct RollingContext<'a> {
pub struct RollingContext {
    bsk: LWEBSK,
    sk: LWESecretKey,
    //xv: Vec<f64>, 
    xv: VectorLWE, 
    //cache: HashMap<Vec<usize>,(f64, i32, &'a VectorLWE)>,
    cache: HashMap<Vec<usize>,(f64, VectorLWE)>,
    size: i32
}

//impl<'a> From<&'a VectorLWE> for RollingContext {
impl<'a> RollingContext {

    // this implementation tests calculating rolling averages for window size is N=2^k 
    // it exploites the strcture of the binary tree by chaching intermediate results
    // padding LWE ciphertexts is simulated by introducing an artificial padding parameter
    // a simulated bootstrap is only carried out when remaining padding reaches 1
    // bootstrap is optional at top level (as it may be effecient to defer)
    // it is intended to test the structure used to caclute rolling ten day T1D scores
    
    pub fn partial_mean_recursion(&mut self, iv: &[usize], l: i32, enc: &Encoder) -> (f64, VectorLWE) {
        // iv is list of indices for which to find average
        // p: is padding parameter which controls how often to rescale
        // l: is level parameter for l>0 rescales all levels, otherwise manual rescale of final 
        //let max_padding: i32 = 2;
        let n = iv.len();
        assert!(n>0);
        
        /*
        match self.cache.get(iv) {
            Some(&result) => {return result;}, //println!("Found in cache for: {:?}",iv); 
            _ => {}, //println!("Didnt find in cache."),
        }
        */
        if self.cache.contains_key(iv) {
            //println!("Found {:?} in cache!",iv);
            let (s, rs) = self.cache.get(iv).unwrap();
            //println!("nb_ciphertexts: {}", rs.nb_ciphertexts);
            return (*s, rs.clone()); 
        }
        
        let m = n/2;
        if n == 1 {
            let idx = iv[0];
            //let val = self.xv[idx];
            let val = self.xv.extract_nth(idx).unwrap();
            let p = val.encoders[0].nb_bit_padding;
            println!("n, m, p: {}, {} {}", n, m, p);
            return (1.0, val.clone());
        }
        assert!(n-m==m);
        let (ss, rr) = self.partial_mean_recursion(&iv[..m], l+1, &enc);
        let (_s, _r) = self.partial_mean_recursion(&iv[m..], l+1, &enc);
        assert!(rr.encoders[0].nb_bit_padding==_r.encoders[0].nb_bit_padding);
        let mut s = 0.5*ss;
        //let mut rs = rr+_r;
        //let mut rs = rr.clone(); // XXX temporaily just pass first
        let mut rs = rr.add_with_padding(&_r).unwrap();
        //
        //let enc_part = Encoder::new(min, max, enc.nb_bit_precision, enc.nb_bit_padding).unwrap();
        //let term1 = (*x1).bootstrap_nth_with_function(&self.bsk, |x| phi * f1(x), &enc_part, 0).unwrap();
        //let term2 = (*x2).bootstrap_nth_with_function(&self.bsk, |x| (1.-phi) * f2(x), &enc_part, 0).unwrap();
        //let res = rr.add_with_padding(&_r).unwrap();
        //
        //let p = pp - 1; // XXX handling of padding
        let p = rs.encoders[0].nb_bit_padding;
        if p == 1 && l > 0 { // bootstrap partial mean
            println!("n, m, p, s: {}, {}, {}, {} - restore padding and scale", n, m, p, s);
            //rs = s*rs; // XXX here is bootstrap happening
            //rs = rs.clone(); // XXX dummy op 
            rs = rs.bootstrap_nth_with_function(&self.bsk, |x| x, &enc, 0).unwrap();
            s = 1.0; 
            //p = max_padding; // XXX handling of padding
        }
        else {
            //println!("n, m, p, s: {}, {}, {}, {}", n, m, p, s);    
        }
        self.cache.insert(iv.to_vec(), (s, rs.clone()));
        self.size += 1;
        return (s, rs);
    }
    
    pub fn rolling_mean_of_2toN(&mut self, w: usize, l: i32) -> (f64, VectorLWE) {
        // w: size of rolling window which must be 2^N
        // l: is level parameter for l>0 rescales all levels, otherwise manual rescale of final 
        //let n = self.xv.len();
        //let n = self.xv.nb_ciphertexts;
        let n = 2*w; // XXX while testing
        let iv: Vec<usize> = (0..n).collect();
        let k = n - w + 1;
        //let mut rv: Vec<f64> = vec![0.0; k];
        let mut rv = VectorLWE::zero(1024, k).unwrap(); // XXX make polymod variable
        let base = w/2;
        let mut scale = 1.0;
        let enc = self.xv.encoders[0].clone();

        for i in 0..k {
            let m = i/base;
            let c = i - m*base;
            let mut v = (i..(i+w)).map(|i| iv[i]).collect::<Vec<_>>();
            v.rotate_right(c);
            let (s, r) = self.partial_mean_recursion(&v, l, &enc);
            //rv[i] = r;
            rv.copy_in_nth_nth_inplace(i, &r, 0).unwrap();
            scale = s;
            println!("{} {} {} {:?} {}",i,m,c,v,self.size);
        }
        return (scale, rv);
    }
}

/*
pub struct ConcreteContext {
    bsk: LWEBSK,
    sk: LWESecretKey
}

fn id(x: f64) -> f64 { x }

impl ConcreteContext {
    
    // this implementation can calculate mean of uneven sized sample (i.e n != 2^k)
    // albeit requires bootstrap at every addition for a total of 2n-1
    // on the other hand it can run a function at leaf nodes e.g. to calculate score
    // which is included in above execution time cost estimate
    // it has no support for rolling averages as it is inteded for hourly T1D scores

    pub fn weighted_mean_of_pair(&mut self, x1: &VectorLWE, x2: &VectorLWE, phi: f64, enc: &Encoder, 
            f1: fn(f64) -> f64, f2: fn(f64) -> f64) -> VectorLWE {
        // phi is weight of x1
        // enc is target encoder
        // f1 and f2 are functions to evaluate on x1 and x2 (set to id for mean of data)
        assert!(phi > 0.0);
        assert!(phi < 1.0); 
        let scale: f64 = if phi > 0.5 {phi} else {1.-phi}; 
        let min = scale*enc.o;
        let max = scale*(enc.o + enc.delta);
        let enc_part = Encoder::new(min, max, enc.nb_bit_precision, enc.nb_bit_padding).unwrap();
        let term1 = (*x1).bootstrap_nth_with_function(&self.bsk, |x| phi * f1(x), &enc_part, 0).unwrap();
        let term2 = (*x2).bootstrap_nth_with_function(&self.bsk, |x| (1.-phi) * f2(x), &enc_part, 0).unwrap();
        let res = term1.add_with_padding(&term2).unwrap();
        return res;
    }       
        
    pub fn weighted_mean_recursion(&mut self, x: &[VectorLWE], enc: &Encoder, 
            f: fn(f64) -> f64) -> (fn(f64) -> f64, VectorLWE) {
        // x is vector of data
        // enc is target encoder
        // f is function to evaluate on leaf node (set to id for mean of data)
        let n = x.len();
        assert!(n>0);
        let m = n/2;
        if n == 1 {
            println!("n, m: {}, {}", n, m);
            return (f, x[0].clone());
        }
        let (f_i, x_i) = self.weighted_mean_recursion(&x[..m], &enc, f);
        let (f_j, x_j) = self.weighted_mean_recursion(&x[m..], &enc, f);
        let phi: f64 = (m as f64)/(n as f64);
        println!("n, m, phi: {}, {}, {}", n, m, phi);
        return (id, self.weighted_mean_of_pair(&x_i, &x_j, phi, &enc, f_i, f_j));
    }
    
    pub fn weighted_mean_of_many(&mut self, x: &[VectorLWE], enc: &Encoder, f: fn(f64) -> f64) -> VectorLWE{
        // x is vector of data
        // enc is target encoder
        // f is function to evaluate on leaf node (set to id for mean of data)
         let (_f, res) = self.weighted_mean_recursion(&x, &enc, f);
        return res.bootstrap_nth_with_function(&self.bsk, |x| x, &enc, 0).unwrap();           
    }
        
    pub fn bootmap(&mut self, v: &[VectorLWE], f: &dyn Fn(f64) -> f64, enc: &Encoder) -> Vec<VectorLWE> {
        let n = v.len();
        (0..n).map(|i| v[i].bootstrap_nth_with_function(&self.bsk,f,&enc,0).unwrap()).collect::<Vec<_>>()
    }
    
    pub fn encrypt(&mut self, v: &Vec<f64>, enc: &Encoder) -> Vec<VectorLWE> {
        let n = v.len();
        (0..n).map(|i| VectorLWE::encode_encrypt(&self.sk, &[v[i]], &enc).unwrap()).collect::<Vec<_>>()
    }
}
*/

fn main() -> Result<(), CryptoAPIError> {

    let data_path = "data/CGM_p77_24h";
    let path = "keys/80_1024_1";
    let mut base_log: usize = 6;
    let mut level: usize = 4; 
    
    println!("setting parameters ..."); 
    let args: Vec<String> = env::args().collect();
    if args.len() == 3 {
        base_log =  args[1].parse().unwrap();
        level = args[2].parse().unwrap();
    }
    println!("base_log {}", base_log);
    println!("level {}\n", level);    
    let path = format!("{}_{}_{}",path,base_log,level);

    
    println!("loading keys and data ... ");
    println!("keys: {}",path);
    let sk0_LWE_path = format!("{}/sk0_LWE.json",path);
    let bsk00_path = format!("{}/bsk00_LWE.json",path);
    let encfile = format!("{}_{}_{}.enc",data_path,base_log,level);
    println!("data: {}",encfile);
    let now = Instant::now();
    let mut context = RollingContext{
        bsk: LWEBSK::load(&bsk00_path),
        sk: LWESecretKey::load(&sk0_LWE_path).unwrap(),
        xv: VectorLWE::load(&encfile).unwrap(),
        cache: HashMap::new(),
        size: 0
    };    
    let load_time = now.elapsed().as_millis();
    println!("data enc: {:?}", context.xv.encoders[0]);
    println!("load keys and data (size {}) took {} ms\n", context.xv.nb_ciphertexts, load_time);
    
    /*
    println!("open file to log stats ... ");    
    //let savefile = format!("{}_{}_{}_cgm.enc",data_path,base_log,level);
    //let mut file = OpenOptions::new()
    //    .append(true)
    //    .open("calc_hourly_stats.txt")
    //    .unwrap();    
    */
    
    println!("calcuate rolling averages ... ");
    
    let w = 8;
    
    let (_s, av) = context.rolling_mean_of_2toN(w,1);
    println!("av: {:?}", av.decrypt_decode(&context.sk).unwrap());
    
    /*    
    let mut cgm = VectorLWE::zero(1024, k).unwrap();
    let mut tau: Vec<u128> = Vec::new();
    
    for i in 0..k {
        let mut x: Vec<VectorLWE> = Vec::new();
        println!("i: {}",i);
        for j in 0..w {
            x.push(ct.extract_nth(i*w+j).unwrap());
        }
        let now = Instant::now();
        //let avg = ct.extract_nth(i).unwrap(); // for dummy run
        let avg = context.weighted_mean_of_many(&x, &encCGM, id);
        let step_time = now.elapsed().as_millis();
        tau.push(step_time);
        println!("window size {} took {} ms", w, step_time);
        cgm.copy_in_nth_nth_inplace(i, &avg, 0).unwrap();
        //println!("mean hourly CGM* {:?}", avg.decrypt_decode(&context.sk).unwrap());
        //let avg = context.weighted_mean_of_many(&x, &encCGM, score_GVP);
        //println!("mean hourly GVP* {:?}", avg.decrypt_decode(&context.sk).unwrap());
        //let avg = context.weighted_mean_of_many(&x, &enc100, score_IR);
        //println!("mean hourly TIR* {:?}", avg.decrypt_decode(&context.sk).unwrap());
        let msg = format!("{} {}", savefile, step_time);
        println!("{}", msg);
        writeln!(file, "{}", msg).unwrap();    
    }
    println!("write cgm {}", savefile);
    cgm.save(&savefile).unwrap();    

    println!("average CGM* per bucket{:?}", cgm.decrypt_decode(&context.sk).unwrap());
    println!("time consumed per bucket {:?}", tau);
    */

       
    Ok(())
}
