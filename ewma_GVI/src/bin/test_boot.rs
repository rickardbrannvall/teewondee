#![allow(non_snake_case)]
use concrete::*;
use std::env;

use concrete_npe as npe;
use concrete_commons::numeric::Numeric;

use std::fs::OpenOptions;
use std::io::prelude::*;

fn main() -> Result<(), CryptoAPIError> {

    let path = "keys/80_1024_1";
    let mut base_log: usize = 5;
    let mut level: usize = 3; 
    let args: Vec<String> = env::args().collect();
    if args.len() == 3 {
        base_log =  args[1].parse().unwrap();
        level = args[2].parse().unwrap();
    }
    println!("base_log: {}", base_log);
    println!("level: {}", level);    
    let path = format!("{}_{}_{}",path,base_log,level);
    println!("key path: {}", path);    

    let mut file = OpenOptions::new()
        .append(true)
        .open("boot_stats.txt")
        .unwrap();

    //if let Err(e) = writeln!(file, "A new line!") {
    //    eprintln!("Couldn't write to file: {}", e);
    //}    
    
    println!("loading LWE sk 0... \n");
    let sk0_LWE_path = format!("{}/sk0_LWE.json",path);
    let sk0 = LWESecretKey::load(&sk0_LWE_path).unwrap();    
        
    // create an encoder
    let enc = Encoder::new(0., 8., 3, 2)?;
            
    let x: Vec<f64> = vec![4.0]; // initial value for data generating process
    println!("x {:?}\n", x);

    let x0 = VectorLWE::encode_encrypt(&sk0, &x, &enc)?;  
    println!("x* {:?}",x0.decrypt_decode(&sk0).unwrap());
    x0.pp();    

    let v0 = &x0.variances[0];
    println!("v0 {:?}", v0);    

    let n0 = npe::nb_bit_from_variance_99(*v0, <Torus as Numeric>::BITS as usize);
    println!("n0 {:?}", n0);    
    
    let s0 = <Torus as Numeric>::BITS - n0;
    println!("s0 {:?}", s0);    

    println!("loading BSK 00... \n");
    let bsk00_path = format!("{}/bsk00_LWE.json",path);
    let bsk00 = LWEBSK::load(&bsk00_path);
    
    let x1 = x0.bootstrap_nth_with_function(&bsk00, |x| x, &enc,0).unwrap();
    
    let rs = x1.decrypt_decode(&sk0);
    let rs = match rs {
        Ok(data) => data,
        Err(_error) => {
            println!("{} NA", path);
            writeln!(file, "{} NA", path).unwrap();
            return Ok(())
        }
    };    
    
    println!("f(x*) {:?}", rs);
    x1.pp();
    
    let v1 = &x1.variances[0];
    println!("v1 {:?}", v1);

    let n1 = npe::nb_bit_from_variance_99(*v1, <Torus as Numeric>::BITS as usize);
    println!("n1 {:?}", n1);  

    let s1 = <Torus as Numeric>::BITS - n1;
    println!("s1 {:?}", s1); 
    
    println!("{} {}", path, s1);
    writeln!(file, "{} {}", path, s1).unwrap();
        
    Ok(())
}
