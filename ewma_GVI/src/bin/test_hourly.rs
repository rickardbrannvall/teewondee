#![allow(non_snake_case)]
use concrete::*;
use std::env;

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

fn score_IR(x: f64) -> f64 { if (x >= 70.) && (x <= 180.) {100.0} else {0.0} }
//fn score_70(x: f64) -> f64 { if x < 70. {100.0} else {0.0} }
//fn score_54(x: f64) -> f64 { if x < 54. {100.0} else {0.0} }
//fn score_GVP(dy: f64) -> f64 {(1.0 + (dy/5.0).powi(2)).sqrt() - 1.0}

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
    2021-01-01 13:03:39,191.0
    2021-01-01 13:08:39,186.0
    2021-01-01 13:13:39,184.0
    2021-01-01 13:18:39,183.0
    2021-01-01 13:23:39,179.0
    2021-01-01 13:28:39,177.0
    2021-01-01 13:33:39,167.0
    2021-01-01 13:38:39,163.0
    2021-01-01 13:43:39,156.0
    2021-01-01 13:48:39,153.0
    2021-01-01 13:53:39,150.0
    2021-01-01 13:58:39,157.0
    */
    // time in range 8/12 approx 67%

    // test mean_of_many
    
    let xv: Vec<f64> = vec![191.,186.,184.,183.,179.,177.,167.,163.,156.,153.,150.,157.];

    let cv = context.encrypt(&xv, &encCGM);
    println!("cv[0]* {:?}", cv[0].decrypt_decode(&context.sk).unwrap());
    cv[0].pp(); 
        
    /*
    let ir = context.bootmap(&cv, &score_IR, &enc100);
    println!("ir[0]* {:?}", ir[0].decrypt_decode(&context.sk).unwrap());
    ir[0].pp(); 

    let h70 = context.bootmap(&cv, &score_70, &enc100);
    println!("h70[0]* {:?}", h70[0].decrypt_decode(&context.sk).unwrap());
    h70[0].pp(); 

    let h54 = context.bootmap(&cv, &score_54, &enc100);
    println!("h54[0]* {:?}", h54[0].decrypt_decode(&context.sk).unwrap());
    h54[0].pp(); 
    */
    
    let avg = context.weighted_mean_of_many(&cv, &encCGM, id);
    println!("mean hourly CGM* {:?}", avg.decrypt_decode(&context.sk).unwrap());
    avg.pp(); 
    
    let tst = xv.iter().sum::<f64>() / xv.len() as f64;
    println!("check value: {}",tst);

    let avg = context.weighted_mean_of_many(&cv, &enc100, score_IR);
    println!("mean hourly TIR* {:?}", avg.decrypt_decode(&context.sk).unwrap());
    avg.pp(); 
    
    Ok(())
}
