#![allow(non_snake_case)]
use concrete::*;
use std::env;

pub struct ConcreteContext {
    bsk: LWEBSK,
    sk: LWESecretKey
}

impl ConcreteContext {
     
    pub fn mean_of_pair(&mut self, x1: &VectorLWE, x2: &VectorLWE, phi: f64, enc: &Encoder) -> VectorLWE {
        // assert phi is in range [0.0,1.0] 
        let scale: f64 = if phi > 0.5 {phi} else {1.-phi}; 
        let min = scale*enc.o;
        let max = scale*(enc.o + enc.delta);
        let enc_part = Encoder::new(min, max, enc.nb_bit_precision, enc.nb_bit_padding).unwrap();
        let term1 = (*x1).bootstrap_nth_with_function(&self.bsk, |x| 0.5 * x, &enc_part, 0).unwrap();
        let term2 = (*x2).bootstrap_nth_with_function(&self.bsk, |x| 0.5 * x, &enc_part, 0).unwrap();
        let res = term1.add_with_padding(&term2).unwrap();
        return res;
    }  
    
    pub fn mean_recursion(&mut self, x: &[VectorLWE], enc: &Encoder) -> VectorLWE{
        // assert n>0
        let n = x.len();
        let m = n/2;
        if n == 1 {
            println!("n, m: {}, {}", n, m);
            return x[0].clone();
        }
        let x_i = self.mean_recursion(&x[..m], &enc);
        let x_j = self.mean_recursion(&x[m..], &enc);
        let phi: f64 = (m as f64)/(n as f64);
        println!("n, m, phi: {}, {}, {}", n, m, phi);
        return self.mean_of_pair(&x_i, &x_j, phi, &enc);
    }
    
    pub fn mean_of_many(&mut self, x: &[VectorLWE], enc: &Encoder) -> VectorLWE{
        let res = self.mean_recursion(&x, &enc);
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


fn main() -> Result<(), CryptoAPIError> {

    let path = "keys/80_1024_1";
    let mut base_log: usize = 6;
    let mut level: usize = 4; 
    let args: Vec<String> = env::args().collect();
    if args.len() == 3 {
        base_log =  args[1].parse().unwrap();
        level = args[2].parse().unwrap();
    }
    println!("base_log {}", base_log);
    println!("level {}", level);    
    let path = format!("{}_{}_{}",path,base_log,level);
    println!("key path {}", path);    
    
    //println!("loading LWE sk 0... \n");
    let sk0_LWE_path = format!("{}/sk0_LWE.json",path);
    //let sk0 = LWESecretKey::load(&sk0_LWE_path).unwrap();    
    let bsk00_path = format!("{}/bsk00_LWE.json",path);
    //let bsk00 = LWEBSK::load(&bsk00_path);

    let encCGM = Encoder::new(0., 200., 3, 2).unwrap();
    let enc100 = Encoder::new(0., 100., 3, 2).unwrap();
    //let enc10 = Encoder::new(0., 10., 3, 2).unwrap();
    //let enc5 = Encoder::new(0., 5., 3, 2).unwrap();
    
    let mut context = ConcreteContext{
        bsk: LWEBSK::load(&bsk00_path),
        sk: LWESecretKey::load(&sk0_LWE_path).unwrap()    
    };    

    /*
    2021-01-01 12:18:40,189.0
    2021-01-01 12:23:39,195.0
    2021-01-01 12:28:40,193.0
    2021-01-01 12:33:39,199.0
    2021-01-01 12:38:39,193.0
    2021-01-01 12:43:39,195.0
    2021-01-01 12:48:39,198.0
    2021-01-01 12:53:39,202.0
    2021-01-01 12:58:39,195.0
    2021-01-01 13:03:39,191.0
    */

    // test mean_of_many
    
    let xv: Vec<f64> = vec![179.0,175.0,183.0,189.0,183.0,185.0,188.0,182.0,175.0,171.0];

    let cv = context.encrypt(&xv, &encCGM);
    println!("cv[0]* {:?}", cv[0].decrypt_decode(&context.sk).unwrap());
    cv[0].pp(); 
        
    fn score_IR(x: f64) -> f64 { if (x >= 70.) && (x <= 180.) {100.0} else {0.0} }
    let ir = context.bootmap(&cv, &score_IR, &enc100);
    println!("ir[0]* {:?}", ir[0].decrypt_decode(&context.sk).unwrap());
    ir[0].pp(); 

    fn score_70(x: f64) -> f64 { if x < 70. {100.0} else {0.0} }
    let h70 = context.bootmap(&cv, &score_70, &enc100);
    println!("h70[0]* {:?}", h70[0].decrypt_decode(&context.sk).unwrap());
    h70[0].pp(); 

    fn score_54(x: f64) -> f64 { if x < 54. {100.0} else {0.0} }
    let h54 = context.bootmap(&cv, &score_54, &enc100);
    println!("h54[0]* {:?}", h54[0].decrypt_decode(&context.sk).unwrap());
    h54[0].pp(); 
    
    //let avg = context.mean_of_many(&cv, &encCGM);
    let avg = context.mean_of_many(&ir, &enc100);
    println!("avg* {:?}", avg.decrypt_decode(&context.sk).unwrap());
    avg.pp(); 
    
    let tst = xv.iter().sum::<f64>() / xv.len() as f64;
    println!("tst: {}",tst);

    Ok(())
}
