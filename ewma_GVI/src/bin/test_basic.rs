#![allow(non_snake_case)]
#![allow(dead_code)]

use concrete::*;

fn main() -> Result<(), CryptoAPIError> {
    
    // TFHE implementation
    
    let path = "keys";
    //let path = "/home/jovyan/shared/teewondee/ewma_GVI/keys";
    
    // load LWE keys
    
    println!("loading LWE key... \n");
    let sk0_LWE_path = format!("{}/sk0_LWE.json",path);
    //println!("{}",sk0_LWE_path);
    let sk0 = LWESecretKey::load(&sk0_LWE_path).unwrap();    

    /*
    // set-up for bootstrapping
    
    println!("Load Bootstrapping Key 00 ... \n");
    let bsk00_path = format!("{}/bsk00_LWE.json", path);
    let bsk00 = LWEBSK::load(&bsk00_path);    
    let enc_cmp = Encoder::new(0.0, 1.0, 1, 2).unwrap();
    */
    
    // create an encoder
    println!("create an encoder... \n");
    let enc = Encoder::new(0., 10., 6, 4)?;

    let m0: Vec<f64> = vec![2.54];
    println!("plaintext value {:?}\n", m0);
    
    let c0 = VectorLWE::encode_encrypt(&sk0, &m0, &enc)?;  
    println!("encrypted value {:?}", c0.decrypt_decode(&sk0).unwrap());
    c0.pp();
    
    let constants: Vec<f64> = vec![1.0];
    
    let mut ct = VectorLWE::encode_encrypt(&sk0, &m0, &enc)?;  
    ct.add_constant_static_encoder_inplace(&constants)?; 
    println!("add constant one {:?}", ct.decrypt_decode(&sk0).unwrap());
    ct.pp();   

    let mut ct = VectorLWE::encode_encrypt(&sk0, &m0, &enc)?;  
    ct.add_with_padding_inplace(&c0)?;
    println!("add with padding {:?}", ct.decrypt_decode(&sk0).unwrap());
    ct.pp();       

    ct = VectorLWE::encode_encrypt(&sk0, &m0, &enc)?;  
    ct.add_with_new_min_inplace(&c0, &vec![0.0])?;
    println!("add with new min {:?}", ct.decrypt_decode(&sk0).unwrap());
    ct.pp();     

    let max_constant: f64 = 1.0;
    let nb_bit_padding = 4;

    ct = VectorLWE::encode_encrypt(&sk0, &m0, &enc)?;  
    ct.mul_constant_with_padding_inplace(&constants, max_constant, nb_bit_padding)?;
    println!("mul constant one {:?}", ct.decrypt_decode(&sk0).unwrap());
    ct.pp();      

    ct = VectorLWE::encode_encrypt(&sk0, &m0, &enc)?;  
    ct.opposite_nth_inplace(0).unwrap();
    println!("negation of val {:?}", ct.decrypt_decode(&sk0).unwrap());
    ct.pp();     
    
    Ok(())
}
