#![allow(non_snake_case)]
use concrete::*;
use std::env;

pub struct ConcreteContext {
    enc_full: Encoder,
    enc_half: Encoder,
    bsk: LWEBSK,
    sk: LWESecretKey
}

impl ConcreteContext {
    pub fn mean_of_pair(&mut self, x1: &VectorLWE, x2: &VectorLWE) -> VectorLWE {
        let term1 = (*x1).bootstrap_nth_with_function(&self.bsk, |x| 0.5 * x, &self.enc_half, 0).unwrap();
        //println!("term1* {:?}", term1.decrypt_decode(&sk0).unwrap());
        //term1.pp();

        let term2 = (*x2).bootstrap_nth_with_function(&self.bsk, |x| 0.5 * x, &self.enc_half, 0).unwrap();
        //println!("term2* {:?}", term2.decrypt_decode(&sk0).unwrap());
        //term2.pp();

        let res = term1.add_with_padding(&term2).unwrap();
        //println!("ewma* {:?}", m1.decrypt_decode(&sk0).unwrap());
        //m1.pp();   
        return res;
    }  
    
    pub fn mean_of_many(&mut self, x: &[VectorLWE]) -> VectorLWE{
        let n = x.len();
        let m = n/2;
        println!("n,m: {},{}", n, m);
        if n > 1 {
            let x_i = self.mean_of_many(&x[..m]);
            let x_j = self.mean_of_many(&x[m..]);
            return self.mean_of_pair(&x_i, &x_j);
        }
        else {
            return x[0].clone();
        }
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

    let mut context = ConcreteContext{
        enc_full: Encoder::new(0., 200., 3, 2).unwrap(),
        enc_half: Encoder::new(0., 100., 3, 2).unwrap(),
        bsk: LWEBSK::load(&bsk00_path),
        sk: LWESecretKey::load(&sk0_LWE_path).unwrap()    
    };    
        
    let x1: Vec<f64> = vec![160.0]; // initial value for moving average process
    println!("x1: {:?}\n", x1);
    
    let x2: Vec<f64> = vec![140.0]; // initial value for data generating process
    println!("x2: {:?}\n", x2);

    let c1 = VectorLWE::encode_encrypt(&context.sk, &x1, &context.enc_full)?;  
    println!("x1* {:?}", c1.decrypt_decode(&context.sk).unwrap());
    c1.pp();

    let c2 = VectorLWE::encode_encrypt(&context.sk, &x2, &context.enc_full)?;  
    println!("x2* {:?}",c2.decrypt_decode(&context.sk).unwrap());
    c2.pp();  
        
    let c3 = context.mean_of_pair(&c1, &c2);
    println!("res* {:?}", c3.decrypt_decode(&context.sk).unwrap());
    c3.pp();   
    
    println!("res: {}", 0.5*&x1[0]+0.5*&x2[0]);

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
    
    //let xv: Vec<f64> = vec![189.0,195.0,193.0,199.0,193.0,195.0,198.0,202.0,195.0,191.0];
    let xv: Vec<f64> = vec![189.0,195.0,193.0,199.0,193.0,195.0,198.0,202.0];
    let n = xv.len();
    
    
    let cv = (0..n).map(|i| VectorLWE::encode_encrypt(&context.sk, &[xv[i]], &context.enc_full).unwrap())
                    .collect::<Vec<_>>();
    
    /*
    let cv = VectorLWE::encode_encrypt(&context.sk, &xv, &context.enc_full).unwrap();
    let mut cv = vec![c1.clone(); n];
    for i in 0..n {
        cv[i] = x.extract_nth(i).unwrap();
    }
    */
    
    let avg = context.mean_of_many(&cv);
    println!("avg* {:?}", avg.decrypt_decode(&context.sk).unwrap());
    avg.pp(); 
    
    let tst = xv.iter().sum::<f64>() / xv.len() as f64;
    println!("tst: {}",tst);

    Ok(())
}
