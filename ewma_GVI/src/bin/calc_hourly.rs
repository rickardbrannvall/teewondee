#![allow(non_snake_case)]
use concrete::*;

use std::env;
use std::time::{Instant}; // Duration, 
use std::fs::OpenOptions;
use std::io::prelude::*;

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
            //println!("n, m: {}, {}", n, m);
            return (f, x[0].clone());
        }
        let (f_i, x_i) = self.weighted_mean_recursion(&x[..m], &enc, f);
        let (f_j, x_j) = self.weighted_mean_recursion(&x[m..], &enc, f);
        let phi: f64 = (m as f64)/(n as f64);
        //println!("n, m, phi: {}, {}, {}", n, m, phi);
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


/*
fn score_GVP(dy: f64) -> f64 {
    let ceil = 2.;
    let gvp = (1.0 + (dy/5.0).powi(2)).sqrt() - 1.0;
    let res = if gvp < ceil {gvp} else {ceil};
    return 100.*res;
}
*/
//fn score_IR(x: f64) -> f64 { if (x >= 70.) && (x <= 180.) {100.0} else {0.0} }
//fn score_70(x: f64) -> f64 { if x < 70. {100.0} else {0.0} }
//fn score_54(x: f64) -> f64 { if x < 54. {100.0} else {0.0} }

fn main() -> Result<(), CryptoAPIError> { // calc_hourly keys data

    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        println!("format: calc_hourly keys data");
        println!("example calc_hourly 80_1024_1_6_4 CGM_p77_24h_2_3_400");
        return Ok(());
    }
    
    println!("\nsetting parameters ..."); 
    let mut keys = "80_1024_1_6_4";
    let mut data = "CGM_p77_24h_2_3_400";
    if args.len() == 3 {
        keys = &args[1];
        data = &args[2];
    }
    println!("assume: calc_hourly {} {}", keys, data);
    
    let mut keysize = 1024;
    if keys.contains("2048") {
        keysize = 2048;
    }
    println!("key size: {}", keysize);
    

    let sk0_LWE_path = format!("keys/{}/sk0_LWE.json",keys);
    let bsk00_path = format!("keys/{}/bsk00_LWE.json",keys);
    let datafile = format!("data/{}/{}.enc",keys,data);
    let savefile = format!("data/{}/{}_CGM_hourly.enc",keys,data);
    let log_file = format!("hourly_stats.txt");
    
    
    println!("loading data ...");
    let now = Instant::now();
    println!("read {}",datafile);
    let ct = VectorLWE::load(&datafile).unwrap();
    let key_load_time = now.elapsed().as_millis();
    println!("{} length {} ({} ms)\n", datafile, ct.nb_ciphertexts, key_load_time);

    
    println!("loading keys ... ");
    let now = Instant::now();
    let mut context = ConcreteContext{
        bsk: LWEBSK::load(&bsk00_path),
        sk: LWESecretKey::load(&sk0_LWE_path).unwrap()
    };    
    let key_load_time = now.elapsed().as_millis();
    println!("{} ({} ms)\n", keys, key_load_time);


    println!("open file to log stats ... \n");    
    let mut file = OpenOptions::new()
        .append(true)
        .open(log_file)
        .unwrap();    

    
    println!("calcuate averages ... ");
    let encCGM = ct.encoders[0].clone();
    //let enc100 = Encoder::new(0., 100., 3, 2).unwrap();
    
    let w = 12;
    //let n = 24*12;
    let n = ct.nb_ciphertexts;
    let k = (n+w/2)/w;
    
    let mut cgm = VectorLWE::zero(keysize, k).unwrap();
    let mut tau: Vec<u128> = Vec::new();
    
    for i in 0..k {
        let mut x: Vec<VectorLWE> = Vec::new();
        println!("i: {}",i);
        for j in 0..w {
            let m = i*w+j;
            if m >= n {
                break;
            }
            x.push(ct.extract_nth(m).unwrap());
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
       
    Ok(())
}
