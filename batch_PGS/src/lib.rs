#![allow(dead_code, unused_variables, unused_variables, dead_code, unused_mut, unused_imports, non_snake_case, unused_assignments, unused_must_use)]
use std::fs::File;
use std::io::prelude::*;
use std::str::FromStr;
use std::path::*;
use std::thread;
use std::time::Duration;

use concrete::*;
use indicatif::ProgressBar;


//-----         DATA LOADNING           -----//
pub fn load_data(file_name: &str, datapoints: usize) -> Vec<f64>{
        /* Open file */
        let mut file = File::open(file_name).expect("Can't open file!");
    
        /* Load file contents */
        let mut contents = String::new();
    
        file.read_to_string(&mut contents)
                .expect("Oops cannot read file...");
    
        /* Get rows of data */
        let str_rows: Vec<&str> = contents.trim().split('\n').collect();
    
        let mut data: Vec<f64> = vec![0.; datapoints];
        for i in 0..datapoints{
                data[i] = f64::from_str(str_rows[i]).unwrap();
        }
        return data
}
/*
pub trait VectorLWERmPadding{
    fn remove_padding_inplace(&mut self, nb: usize);
}
impl VectorLWERmPadding for VectorLWE{
    fn remove_padding_inplace(&mut self, nb: usize) -> (){
        //for (self_enc, self_var, self_ct) = izip!(self.encoders.iter(), self.variance.iter(), self.ciphertext.iter())

            // check that te input encoder has at least 1 bit of padding
        for i in 0..self.nb_ciphertexts{
                if self.encoders[i].nb_bit_padding < nb {
                    //return Err(NotEnoughPaddingError!(self.encoders[i].nb_bit_padding, nb));
                    panic!("Not enough padding");
                }
        }

            // shift of nb bits to the left
            self.ciphertexts.as_mut_tensor().update_with_scalar_shl(&nb);

            // correction of the encoder
        for i in 0..self.nb_ciphertexts{
            self.encoders[i].nb_bit_padding -= nb;
        }

            // call to the NPE to estimate the new variance
        for i in 0..self.nb_ciphertexts{
            let coeff: Torus = 1 << nb;
            self.variances[i] = npe::LWE::single_scalar_mul(self.variances[i], coeff);
        }

            // update the encoder precision based on the variance
        for i in 0..self.nb_ciphertexts{
            self.encoders[i].update_precision_from_variance(self.variances[i]).unwrap();
        }
    }
}
*/

fn sum_N(x: &VectorLWE) -> VectorLWE{//,n: &usize, sk: &LWESecretKey) -> VectorLWE{
    let mut y = x.clone();
    let n = (x.nb_ciphertexts as f32).log2() as usize;
    let padd = x.encoders[0].nb_bit_padding;
    let mut ct_1: VectorLWE;
    let mut ct_2: VectorLWE;
    
    for i in 0..(n as usize){
        if ((padd as i32) - (n as i32) <= 0) && (y.encoders[0].nb_bit_padding == 1){
            println!("Not enough padding!");
            return y;
        }else{
            let N = u32::pow(2, (n-i-1) as u32) as usize;
            let mut tmpVec = VectorLWE::zero(x.dimension, N).unwrap();
            for j in 0..N{
                ct_1 = y.extract_nth(2*j).unwrap();
                ct_2 = y.extract_nth(2*j+1).unwrap();

                ct_1.add_with_padding_inplace(&ct_2).unwrap();

                tmpVec.copy_in_nth_nth_inplace(j, &ct_1, 0).unwrap();
            }
            y = tmpVec.clone();
        }
    }
    return y;    
}

fn in_range(x: f64, upper: f64, lower: f64) -> f64{
    if x < upper && x > lower{
        1.
    }else{
        0.
    }
}

fn calc_l(x: f64) -> f64{
    f64::min(f64::sqrt(x.powf(2.0) + 25.), 32.)
}

fn F_GVP(x: f64) -> f64{
    1.+9./(1.+f64::exp(-0.049*(x-65.47)) )
}

fn F_MG(x: f64) -> f64{
    1.+9./(1.+f64::exp(0.1139*(x-72.08)) )+9./(1.+f64::exp(-0.09195*(x-157.57)) )
}

fn F_PTIR(x: f64) -> f64{
    1.+9./(1.+f64::exp(0.0833*(x-55.04)))
}

fn F_H70(x: f64) -> f64{
    if x <= 7.65 {
        0.5714*x + 0.625
    }else{
        5.
    }
}

fn F_H54(x: f64) -> f64{
    0.5+4.5*(1.-f64::exp(-0.81093*x))
}

fn glycemic_variability_precentage(x: &VectorLWE, keys: (&LWESecretKey, &LWESecretKey, &LWEBSK, &LWEKSK), precision: usize) -> VectorLWE{
    println!(" \n \n \n//---- Glycemic Variability Precentage ----//");
    let (sk0, sk1, bsk01, ksk10) = keys;
    let n_u = x.nb_ciphertexts;
    let n = n_u as f64;
    
    let mut diff = x.clone();
    let mut L = VectorLWE::zero(bsk01.polynomial_size, n_u).unwrap();
    let bts = VectorLWE::zero(bsk01.polynomial_size, 1).unwrap();
    
    let mut ct_1 = x.extract_nth(0).unwrap();
    let mut ct_2 = x.extract_nth(0).unwrap();
    ct_1.sub_with_padding_inplace(&ct_2).unwrap();
    ct_1.pp();
    diff.copy_in_nth_nth_inplace(0, &ct_1, 0).unwrap();
    
    for i in 0..n_u-1{
        ct_1 = x.extract_nth(i+1).unwrap();
        ct_2 = x.extract_nth(i).unwrap();
        ct_1.sub_with_padding_inplace(&ct_2).unwrap();
        diff.copy_in_nth_nth_inplace(i+1, &ct_1, 0).unwrap();
    }
    
    let enc = Encoder::new(4.9, 32.4, precision, ((n_u as f64).log2() as usize)+1).unwrap();
    
    let pb = ProgressBar::new(n_u as u64);
    for i in 0..n_u{
        let bts = diff.bootstrap_nth_with_function(&bsk01, |x| calc_l(x), &enc, i).unwrap();
        //let bts = diff.bootstrap_nth_with_function(&bsk01, |x| f64::min(f64::abs(x)/10., 9.), &enc, i).unwrap();
        L.copy_in_nth_nth_inplace(i, &bts, 0).unwrap();
        pb.inc(1);
        thread::sleep(Duration::from_millis(5));
    }
    //(L.extract_nth(0).unwrap()).pp();
    pb.finish_with_message("Done Calulationg L's");
    //println!("{}, {:?}, {}, {}", L.nb_ciphertexts, L.encoders[0].delta, L.encoders[0].get_min(), L.encoders[0].get_granularity());
    
    //println!("{:.2?}", L.decrypt_decode(&sk1).unwrap());

    L = sum_N(&L);
    let L = L.keyswitch(&ksk10).unwrap();
    //println!("L/L0 = {}", L.decrypt_decode(&sk0).unwrap()[0]/(5.*n));
    //println!(" 100*(L/L0 - 1) = {}", 100.*(L.decrypt_decode(&sk0).unwrap()[0] /(5.*n) - 1.) );
    //println!("F_GVP( 100*(L/L0 - 1) ) = {}", F_GVP(100.*(L.decrypt_decode(&sk0).unwrap()[0] /(5.*n) - 1.)));

    let enc = Encoder::new(0., 10., precision, 3).unwrap();
    let gvp = L.bootstrap_nth_with_function(&bsk01, |x| F_GVP(100.*(x/(5.*n) - 1.)), &enc, 0).unwrap();

    //println!("F_GVP({}) = {}", 100.*(L.decrypt_decode(&sk0).unwrap()[0]/(5.*n) - 1.), F_GVP(100.*(L.decrypt_decode(&sk0).unwrap()[0]/(5.*n)-1.)));
    return gvp;
}

fn mean_glucose(x: &VectorLWE, keys:(&LWESecretKey, &LWESecretKey, &LWEBSK, &LWEKSK), precision: usize) -> VectorLWE{
    println!(" \n //---- Mean Glucose ----//");
    let (sk0, sk1, bsk01, ksk10) = keys;
    let n = x.nb_ciphertexts as f64;
    let enc = Encoder::new(0., 10., precision, 3).unwrap();
    
    let mut mean = sum_N(x);
    let mg = mean.bootstrap_nth_with_function(&bsk01, |x| F_MG(x/n), &enc, 0).unwrap();
    //println!("{}", mg.decrypt_decode(&sk0).unwrap()[0]/n);    
    //println!("F_MG({}) = {}", mean.decrypt_decode(&sk0).unwrap()[0]/n, F_MG(mean.decrypt_decode(&sk0).unwrap()[0]/n) );
    return mg;
}

fn positive_time_in_range(x: &VectorLWE, keys:(&LWESecretKey, &LWESecretKey, &LWEBSK, &LWEKSK), precision: usize) -> VectorLWE{
    println!(" \n \n //---- Positive Time in Range ----//");
    let (sk0, sk1, bsk01, ksk10) = keys;
    
    let n_u = x.nb_ciphertexts;
    let n = n_u as f64;
    let mut tir = VectorLWE::zero(bsk01.polynomial_size, n_u).unwrap();
    
    let enc = Encoder::new(0., 1., precision, ((n_u as f64).log2() as usize)+1).unwrap();
    let pb = ProgressBar::new(n_u as u64);
    for i in 0..n_u{
        let bts = x.bootstrap_nth_with_function(&bsk01, |x| in_range(x, 180., 70.), &enc, i).unwrap();
        tir.copy_in_nth_nth_inplace(i, &bts, 0).unwrap();
        pb.inc(1);
        thread::sleep(Duration::from_millis(5));
    }
    pb.finish_with_message("Done Calulationg L's");
    
    let tir = sum_N(&tir);
    let avg_tir = tir.decrypt_decode(&sk1).unwrap()[0]/n;
    let tir = tir.keyswitch(&ksk10).unwrap();
    
    let enc = Encoder::new(0., 10., precision, 3).unwrap();
    let ptir = tir.bootstrap_nth_with_function(&bsk01, |x| F_PTIR( x/n ), &enc, 0).unwrap();
    //println!("{:?}", ptir.decrypt_decode(&sk1).unwrap());
    //println!("F_PTIR({}) = {}", tir.decrypt_decode(&sk0).unwrap()[0]/n, F_PTIR(tir.decrypt_decode(&sk0).unwrap()[0]/n));
    
    return ptir;
}

fn hypo_70(x: &VectorLWE, keys:(&LWESecretKey, &LWESecretKey, &LWEBSK, &LWEKSK), precision: usize) -> VectorLWE{
    println!(" \n //---- Hypo Glycemic, 70 ----//");
    let (sk0, sk1, bsk01, ksk10) = keys;
    
    let n_u = x.nb_ciphertexts;
    let n = n_u as f64;
    let mut hypo = VectorLWE::zero(bsk01.polynomial_size, n_u).unwrap();
    
    let enc = Encoder::new(0., 1., precision, ((n_u as f64).log2() as usize)+1).unwrap();
    let pb = ProgressBar::new(n_u as u64);
    for i in 0..n_u{
        let bts = x.bootstrap_nth_with_function(&bsk01, |x| in_range(x, 70., 0.), &enc, i).unwrap();
        hypo.copy_in_nth_nth_inplace(i, &bts, 0).unwrap();
        
        pb.inc(1);
        thread::sleep(Duration::from_millis(5));
    }
    pb.finish_with_message("Done Calulationg L's");
    
    let hypo = sum_N(&hypo);
    let hypo = hypo.keyswitch(&ksk10).unwrap();
    
    let enc = Encoder::new(0., 5., precision, 4).unwrap();
    let hypo70 = hypo.bootstrap_nth_with_function(&bsk01, |x| F_H70(x/n), &enc, 0).unwrap();
    //println!("{:?}", hypo70.decrypt_decode(&sk1).unwrap());
    //println!("F_H70({}) = {}", hypo.decrypt_decode(&sk0).unwrap()[0]/n, F_H70(hypo.decrypt_decode(&sk0).unwrap()[0]/n));
    
    return hypo70;
}

fn hypo_54(x: &VectorLWE, keys:(&LWESecretKey, &LWESecretKey, &LWEBSK, &LWEKSK), precision: usize) -> VectorLWE{
    println!(" \n //---- Hypo Glycemic, 54 ----//");
    let (sk0, sk1, bsk01, ksk10) = keys;
    
    let n_u = x.nb_ciphertexts;
    let n = n_u as f64;
    let mut hypo = VectorLWE::zero(bsk01.polynomial_size, n_u).unwrap();
    
    let enc = Encoder::new(0., 1., precision, ((n_u as f64).log2() as usize)+1).unwrap();
    let pb = ProgressBar::new(n_u as u64);
    for i in 0..n_u{
        let bts = x.bootstrap_nth_with_function(&bsk01, |x| in_range(x, 54., 0.), &enc, i).unwrap();
        hypo.copy_in_nth_nth_inplace(i, &bts, 0).unwrap();
        
        pb.inc(1);
        thread::sleep(Duration::from_millis(5));
    }
    pb.finish_with_message("Done Calulationg L's");
    
    let hypo = sum_N(&hypo);
    let hypo = hypo.keyswitch(&ksk10).unwrap();

    let enc = Encoder::new(0., 5., precision, 4).unwrap();
    let hypo54 = hypo.bootstrap_nth_with_function(&bsk01, |x| F_H54(x/n), &enc, 0).unwrap();
    //println!("{:?}", hypo54.decrypt_decode(&sk1).unwrap());
    //println!("F_H54({}) = {}", hypo.decrypt_decode(&sk0).unwrap()[0]/n, F_H70(hypo.decrypt_decode(&sk0).unwrap()[0]/n));
    return hypo54;
}

fn hypo_glycemic(x: &VectorLWE, keys:(&LWESecretKey, &LWESecretKey, &LWEBSK, &LWEKSK), precision: usize) -> VectorLWE{
    let mut H70 = hypo_70(x, keys, precision);
    let H54 = hypo_54(x, keys, precision);
    
    H70.add_with_padding_inplace(&H54).unwrap();
    return H70;
}

fn personal_glycemic_state(gvp: &VectorLWE, mg: &VectorLWE, ptir: &VectorLWE, hypo: &VectorLWE, keys:(&LWESecretKey, &LWESecretKey, &LWEBSK, &LWEKSK)) -> VectorLWE{
    let (sk0, sk1, bsk01, ksk10) = keys;
    
    let mut ct1 = gvp.add_with_padding(mg).unwrap();
    let ct2 = ptir.add_with_padding(hypo).unwrap();
    
    ct1.add_with_padding_inplace(&ct2).unwrap();
    ct1 = ct1.keyswitch(&ksk10).unwrap();
    
    return ct1;
}

pub fn pgs(data: (&VectorLWE, &VectorLWE, &VectorLWE), keys:(&LWESecretKey, &LWESecretKey, &LWEBSK, &LWEKSK)) ->  (VectorLWE, VectorLWE, VectorLWE, VectorLWE, VectorLWE){
    let (_, _, _, ksk10) = keys;
    
    let p = 6;
    
    let (data_gvp, data_mean, data_tir) = data.clone();
    
    let gvp = glycemic_variability_precentage(data_gvp, keys, p);
    
    let mg = mean_glucose(data_mean, keys, p);
    
    let ptir = positive_time_in_range(data_tir, keys, p);
    
    let hypo = hypo_glycemic(data_tir, keys, p);
    
    let pgs = personal_glycemic_state(&gvp, &mg, &ptir, &hypo, keys);

    return (gvp.keyswitch(&ksk10).unwrap(), mg.keyswitch(&ksk10).unwrap(), ptir.keyswitch(&ksk10).unwrap(), hypo.keyswitch(&ksk10).unwrap(), pgs);
}


pub fn keys(params_lwe: &LWEParams, params_rlwe: &RLWEParams, base_log: usize, nr_lvl: usize, print: bool)->(LWESecretKey, LWESecretKey, LWEBSK, LWEKSK){
	
	let _sk: concrete::LWESecretKey;
	let _sk_rlwe: concrete::RLWESecretKey;
	let _sk_out: concrete::LWESecretKey;
	let _bsk: concrete::LWEBSK;
	let _ksk: concrete::LWEKSK;


	let sk = if Path::new("keys/sk.json").exists(){
		LWESecretKey::load("keys/sk.json").unwrap()
	}else{
		LWESecretKey::new(&params_lwe)
	};
	if Path::new("keys/sk.json").exists(){
	}else{	sk.save("keys/sk.json").unwrap();
	};
	if print {println!(" LWE key loaded/created!\n");} 



	let sk_rlwe = if Path::new("keys/sk_rlwe.json").exists(){
		RLWESecretKey::load("keys/sk_rlwe.json").unwrap()
	}else{
		RLWESecretKey::new(&params_rlwe)
	};
	if Path::new("keys/sk_rlwe.json").exists(){
	}else{	sk_rlwe.save("keys/sk_rlwe.json").unwrap();
	};
	if print {println!(" RLWE key loaded/created!\n");} 



	let sk_out = if Path::new("keys/sk_out.json").exists(){
		LWESecretKey::load("keys/sk_out.json").unwrap()
	}else{
		sk_rlwe.to_lwe_secret_key()
	};
	if Path::new("keys/sk_out.json").exists(){
	}else{	sk_out.save("keys/sk_out.json").unwrap();
	};
	if print {println!(" LWE key loaded/created!(Secret key output)\n");} 


	let bsk = if Path::new("keys/bskey.json").exists(){
		LWEBSK::load("keys/bskey.json")
	}else{
		LWEBSK::new(&sk, &sk_rlwe, base_log, nr_lvl)
	};
	if Path::new("keys/bskey.json").exists(){
	}else{	bsk.save("keys/bskey.json");
	};
	if print {println!(" Bootstrapping key loaded/created!\n");} 
	

	let ksk = if Path::new("keys/kskey.json").exists(){
		LWEKSK::load("keys/kskey.json")
	}else{
		//LWEKSK::new(&sk_out, &sk, base_log, nr_lvl)
        LWEKSK::new(&sk_out, &sk, 2, 9)
	};
	if Path::new("keys/kskey.json").exists(){
	}else{ ksk.save("keys/kskey.json");
	};
	if print {println!(" Keyswitching key loaded/created!\n");} 
	
	
    return (sk, sk_out, bsk, ksk);
}
