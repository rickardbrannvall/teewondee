#![allow(non_snake_case)]
use concrete::*;

fn main() -> Result<(), CryptoAPIError> {

    // note that key generation may take several hours 

    //let sk0_RLWE = RLWESecretKey::new(&RLWE128_1024_1); 
    //let sk0_RLWE = RLWESecretKey::new(&RLWE80_1024_1); 
    //let sk0_RLWE = RLWESecretKey::new(&RLWE80_2048_1); 

    // place keys in common directory
        
    //let rlwe_params0 = RLWE128_1024_1;
    //let path = "keys/128_1024_1";

    let rlwe_params0 = RLWE80_1024_1;
    let path = "keys/80_1024_1";
    
    //let rlwe_params0 = RLWE80_2048_1; // dont use
    //let rlwe_params0: RLWEParams = RLWEParams{polynomial_size: 2048, dimension: 1, log2_std_dev: -60};
    //let path = "keys/std60_2048_1";
         
    println!("Creating basis LWE and RLWE keys ...");
    
    let sk0_RLWE = RLWESecretKey::new(&rlwe_params0);     
    let sk0_RLWE_path = format!("{}/sk0_RLWE.json",path);
    sk0_RLWE.save(&sk0_RLWE_path).unwrap();

    let sk0_LWE = sk0_RLWE.to_lwe_secret_key();
    let sk0_LWE_path = format!("{}/sk0_LWE.json",path);
    sk0_LWE.save(&sk0_LWE_path).unwrap();
    
    
    // bootstrapping keys

    println!("Creating bootstrap key 00 ...");

    let bsk00_path = format!("{}/bsk00_LWE.json",path);

    // dessa bestämmer hur bitar nothing
    let base_log: usize = 5;
    let level: usize = 3;  
    let bsk = LWEBSK::new(&sk0_LWE, &sk0_RLWE, base_log, level);
    bsk.save(&bsk00_path);
            
    Ok(())    
    
}
