#![allow(non_snake_case)]
use concrete::*;

fn main() -> Result<(), CryptoAPIError> {

    let path = "keys";
    
    println!("loading LWE sk 0... \n");
    let sk0_LWE_path = format!("{}/sk0_LWE.json",path);
    let sk0 = LWESecretKey::load(&sk0_LWE_path).unwrap();    
    
    /*
    println!("loading LWE sk 1... \n");
    let sk1_LWE_path = format!("{}/sk1_LWE.json",path);
    let sk1 = LWESecretKey::load(&sk1_LWE_path).unwrap();    

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

    // create an encoder
    let enc = Encoder::new(0., 8., 3, 2)?;
        
    let m: Vec<f64> = vec![6.0]; // initial value for moving average process
    println!("ewma at t=0 {:?}\n", m);
    
    let x: Vec<f64> = vec![4.0]; // initial value for data generating process
    println!("data at t=1 {:?}\n", x);

    let m0 = VectorLWE::encode_encrypt(&sk0, &m, &enc)?;  
    println!("ewma* {:?}", m0.decrypt_decode(&sk0).unwrap());
    m0.pp();

    let x0 = VectorLWE::encode_encrypt(&sk0, &x, &enc)?;  
    println!("data* {:?}",x0.decrypt_decode(&sk0).unwrap());
    x0.pp();    
    
    let phi = 0.5; // this is "discount" factor
    
    println!("loading BSK 00... \n");
    let bsk00_path = format!("{}/bsk00_LWE.json",path);
    let bsk00 = LWEBSK::load(&bsk00_path);
    
    // m_{t+1} = phi*x_{t+1} + (1-phi)*m_t 

    let term2 = x0.bootstrap_nth_with_function(&bsk00, |x| (1.-phi) * x, &enc,0)?;
    term2.pp();
    let term1 = m0.bootstrap_nth_with_function(&bsk00, |x| phi * x, &enc,0)?;
    term1.pp();

    let m1 = term1.add_with_padding(&term2)?;
    println!("ewma* {:?}", m1.decrypt_decode(&sk0).unwrap());
    m1.pp();   
    
    Ok(())
}
