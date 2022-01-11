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

fn main() -> Result<(), CryptoAPIError> {

    let data_path = "data/CGM_p77_24h";
    let path = "keys/80_1024_1";
    let mut base_log: usize = 6;
    let mut level: usize = 4; 
    
    println!("\nsetting parameters ..."); 
    let args: Vec<String> = env::args().collect();
    if args.len() == 3 {
        base_log =  args[1].parse().unwrap();
        level = args[2].parse().unwrap();
    }
    println!("base_log {}", base_log);
    println!("level {}\n", level);    
    let path = format!("{}_{}_{}",path,base_log,level);
    
    
    println!("loading data ...");
    let now = Instant::now();
    let encfile = format!("{}_{}_{}.enc",data_path,base_log,level);
    println!("read {}",encfile);
    let ct = VectorLWE::load(&encfile).unwrap();
    let key_load_time = now.elapsed().as_millis();
    println!("{} length {} ({} ms)\n", encfile, ct.nb_ciphertexts, key_load_time);

    
    println!("loading keys ... ");
    let now = Instant::now();
    let sk0_LWE_path = format!("{}/sk0_LWE.json",path);
    let bsk00_path = format!("{}/bsk00_LWE.json",path);
    let mut context = ConcreteContext{
        bsk: LWEBSK::load(&bsk00_path),
        sk: LWESecretKey::load(&sk0_LWE_path).unwrap()
    };    
    let key_load_time = now.elapsed().as_millis();
    println!("{} ({} ms)\n", path, key_load_time);


    println!("open file to log stats ... \n");    
    let savefile = format!("{}_{}_{}_cgm.enc",data_path,base_log,level);
    let mut file = OpenOptions::new()
        .append(true)
        .open("calc_hourly_stats.txt")
        .unwrap();    

    
    println!("calcuate averages ... ");
    let encCGM = Encoder::new(0., 400., 3, 2).unwrap();
    //let enc100 = Encoder::new(0., 100., 3, 2).unwrap();
    //let enc10 = Encoder::new(0., 10., 3, 2).unwrap();
    //let enc5 = Encoder::new(0., 5., 3, 2).unwrap();
    
    let w = 12;
    let n = 24*12;
    let k = n/w;
    
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

    /*
    // test mean_of_many
    
    let xv: Vec<f64> = vec![191.,186.,184.,183.,179.,177.,167.,163.,156.,153.,150.,157.];

    let cv = context.encrypt(&xv, &encCGM);
    println!("cv[0]* {:?}", cv[0].decrypt_decode(&context.sk).unwrap());
    cv[0].pp(); 
        
    let ir = context.bootmap(&cv, &score_IR, &enc100);
    println!("ir[0]* {:?}", ir[0].decrypt_decode(&context.sk).unwrap());
    ir[0].pp(); 

    let h70 = context.bootmap(&cv, &score_70, &enc100);
    println!("h70[0]* {:?}", h70[0].decrypt_decode(&context.sk).unwrap());
    h70[0].pp(); 

    let h54 = context.bootmap(&cv, &score_54, &enc100);
    println!("h54[0]* {:?}", h54[0].decrypt_decode(&context.sk).unwrap());
    h54[0].pp(); 
    
    let avg = context.weighted_mean_of_many(&cv, &encCGM, id);
    println!("mean hourly CGM* {:?}", avg.decrypt_decode(&context.sk).unwrap());
    avg.pp(); 
    
    let tst = xv.iter().sum::<f64>() / xv.len() as f64;
    println!("check value: {}",tst);

    let avg = context.weighted_mean_of_many(&cv, &encCGM, score_GVP);
    println!("mean hourly GVP* {:?}", avg.decrypt_decode(&context.sk).unwrap());
    avg.pp(); 

    let avg = context.weighted_mean_of_many(&cv, &enc100, score_IR);
    println!("mean hourly TIR* {:?}", avg.decrypt_decode(&context.sk).unwrap());
    avg.pp(); 
    */
       
    Ok(())
}
