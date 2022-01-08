#![allow(dead_code, unused_variables, unused_variables, dead_code, unused_mut, unused_imports, non_snake_case, unused_assignments)]
use std::fs;
use std::path::*;
use concrete::*;
mod lib;
use lib::*;


fn main() {
    let lwe_dim = 750;//512, 1024, 2048];
    let lwe_noise = -29;//-19, -40, -62];
    
    let rlwe_dim = 4096; //512, 1024, 2048];
    let rlwe_noise = -62; //-19, -40, -62];
    
    
    let base_log = 4;
    let lvl = 9;
    
    let lwe_params: LWEParams = LWEParams::new(lwe_dim, lwe_noise);
    let rlwe_params: RLWEParams = RLWEParams{polynomial_size: rlwe_dim, dimension: 1, log2_std_dev: rlwe_noise};
    
    println!("Loading/Creating keys!");
    let (sk0, sk1, bsk01, ksk10) = keys(&lwe_params, &rlwe_params, base_log, lvl, false);
    let all_keys = (&sk0, &sk1, &bsk01, &ksk10);//, &bsk00);
    
    let n: usize = 12;
    let N = u32::pow(2, n as u32) as usize;
    
    let data = load_data("../data/first_two_weeks.csv", N);
    //let data = vec![90., 100., 120., 110.];
    
    let enc_gvp = Encoder::new(0., 400., 11, 2).unwrap();
    let enc_mean = Encoder::new(0., 400., 11, n+1).unwrap();
    let enc_ptir = Encoder::new(0., 400., 11, 1).unwrap();
    
    // Vad om man får dGlucose som data också?
    let enc_data_gvp = VectorLWE::encode_encrypt(&sk0, &data, &enc_gvp).unwrap();
    let enc_data_mean = VectorLWE::encode_encrypt(&sk0, &data, &enc_mean).unwrap();
    let enc_data_ptir = VectorLWE::encode_encrypt(&sk0, &data, &enc_ptir).unwrap();
        
    
    println!("Data loaded and encrypted!");
    
    let (gvp, mg, ptir, hypo, pgs) = pgs((&enc_data_gvp, &enc_data_mean, &enc_data_ptir), all_keys);
    let (gvp, mg) = (gvp.decrypt_decode(&sk0).unwrap(), mg.decrypt_decode(&sk0).unwrap());
    let (ptir, hypo) = (ptir.decrypt_decode(&sk0).unwrap(), hypo.decrypt_decode(&sk0).unwrap());
    println!(" \n \n \n//---- Results ----//");
    
    println!("GVP = {:?}, MG = {:?}, PTIR = {:?}, H = {:?}, PGS = {:?}", gvp, mg, ptir, hypo, pgs.decrypt_decode(&sk0).unwrap());
    
    
}
