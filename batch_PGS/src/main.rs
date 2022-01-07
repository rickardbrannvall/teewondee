#![allow(dead_code, unused_variables, unused_variables, dead_code, unused_mut, unused_imports, non_snake_case, unused_assignments)]
use std::fs;
use std::path::*;
use concrete::*;
mod lib;
use lib::*;


fn in_range(x: f64, upper: f64, lower: f64) -> f64{
    if x < upper && x > lower{
        1.
    }else{
        0.
    }
}


fn main() {
    /*
    let lwe_dim = 1024; //512, 1024, 2048];
    let lwe_noise = -40; //-19, -40, -62];
    
    let rlwe_dim = 2048; //512, 1024, 2048];
    let rlwe_noise = -62; //-19, -40, -62];
    */
    let lwe_dim = 1024; //512, 1024, 2048];
    let lwe_noise = -40; //-19, -40, -62];
    
    let rlwe_dim = 4096; //512, 1024, 2048];
    let rlwe_noise = -62; //-19, -40, -62];
    
    
    let base_log = 4;
    let lvl = 8;
    
    let lwe_params: LWEParams = LWEParams::new(lwe_dim, lwe_noise);
    let rlwe_params: RLWEParams = RLWEParams{polynomial_size: rlwe_dim, dimension: 1, log2_std_dev: rlwe_noise};
    
    let (sk0, sk1, bsk01, ksk10) = keys(&lwe_params, &rlwe_params, base_log, lvl, false);
    let all_keys = (&sk0, &sk1, &bsk01, &ksk10);//, &bsk00);
    //println!("Keys done loading!");
    
    let n: usize = 5;
    let N = u32::pow(2, n as u32) as usize;
    
    let data = load_data("../data/first_week.csv", N);
    //let data = vec![90., 100., 120., 110.];
    
    let enc_gvp = Encoder::new(0., 400., 30, 2).unwrap();
    let enc_mean = Encoder::new(0., 400., 10, n+1).unwrap();
    let enc_ptir = Encoder::new(0., 400., 10, 1).unwrap();
    
    let enc_data_gvp = VectorLWE::encode_encrypt(&sk0, &data, &enc_gvp).unwrap();
    let enc_data_mean = VectorLWE::encode_encrypt(&sk0, &data, &enc_mean).unwrap();
    let enc_data_ptir = VectorLWE::encode_encrypt(&sk0, &data, &enc_ptir).unwrap();
    
    println!("Data loaded and encrypted!");
    
    let (gvp, mg, ptir, hypo, pgs) = pgs((&enc_data_gvp, &enc_data_mean, &enc_data_ptir), all_keys);
    let (gvp, mg) = (gvp.decrypt_decode(&sk0).unwrap(), mg.decrypt_decode(&sk0).unwrap());
    let (ptir, hypo) = (ptir.decrypt_decode(&sk0).unwrap(), hypo.decrypt_decode(&sk0).unwrap());
    println!(" \n \n \n//---- Results ----//");
    
    println!("GVP = {:?}, MG = {:?}, PTIR = {:?}, H = {:?}, PGS = {:?}", gvp, mg, ptir, hypo, pgs.decrypt_decode(&sk0).unwrap());
    
    /*
    let enc_in = Encoder::new(-100., 100., 14, 1).unwrap();
    let enc_out = Encoder::new(5., 40., 5, 12).unwrap();
    
    let x = VectorLWE::encode_encrypt(&sk0, &[0., 2., 4., 10., 15., 25.], &enc_in).unwrap();
    for i in 0..6{
        let y = x.bootstrap_nth_with_function(&bsk01, |x| f64::min(f64::sqrt(x.powf(2.0) + 25.), 35.), &enc_out, i).unwrap();
        println!("{:?}", &y.decrypt_decode(&sk1).unwrap());
    }
    */
    /*
    let encoder_input = Encoder::new(0., 400., 12, 1).unwrap();
    let encoder_output = Encoder::new(0., 1., 3, 1).unwrap();

    // secret keys
    let lwe_dim = 1024; //512, 1024, 2048];
    let lwe_noise = -40; //-19, -40, -62];
    
    let rlwe_dim = 512; //512, 1024, 2048];
    let rlwe_noise = -40; //-19, -40, -62];
    
    let bsk_base = 5;
    let bsk_lvl = 6;
    
    let ksk_base = 2;
    let ksk_lvl = 9;
    
    //let bsk_base = 4;
    //let bsk_lvl = 8;
    
    //let ksk_base = 2;
    //let  ksk_lvl = 9;
    
    
    let lwe_params: LWEParams = LWEParams::new(lwe_dim, lwe_noise);
    let rlwe_params: RLWEParams = RLWEParams{polynomial_size: rlwe_dim, dimension: 2, log2_std_dev: rlwe_noise};
    
    let sk_rlwe = RLWESecretKey::new(&rlwe_params);
    let sk_in = LWESecretKey::new(&lwe_params);
    let sk_out = sk_rlwe.to_lwe_secret_key();

    // bootstrapping key
    let bsk = LWEBSK::new(&sk_in, &sk_rlwe, bsk_base, bsk_lvl);
    let ksk = LWEKSK::new(&sk_out, &sk_in, ksk_base, ksk_lvl);

    // messages
    let message: Vec<f64> = vec![175.];//, 181., 185., 190.];
    
    // encode and encrypt
    let c1 = VectorLWE::encode_encrypt(&sk_in, &message, &encoder_input).unwrap();
    //c1.pp();
    
    // bootstrap
    for i in 0..message.len(){
        let c2 = c1.bootstrap_nth_with_function(&bsk, |x| in_range(x, 180., 70.), &encoder_output, i).unwrap();
        c2.pp();
        
        let c3 = c2.keyswitch(&ksk).unwrap();
        c3.pp();
        
        let c4 = c3.bootstrap_nth_with_function(&bsk, |x| in_range(x, 180., 70.), &encoder_output, 0).unwrap();
        c4.pp();
        
        let c5 = c4.keyswitch(&ksk).unwrap();
        c5.pp();
        
        // decrypt
        let output = c3.decrypt_decode(&sk_in).unwrap();

        println!("before bootstrap: {:?}, after bootstrap: {:?}", message, output);
    }
*/
}
