extern crate lib;
use lib::*;

use concrete::*;

use std::io::prelude::*;
use std::fs::File;

fn write_res(func: Vec<&str>, key: Vec<usize>, mean: Vec<Vec<f64>>, std: Vec<Vec<f64>>) -> (){

    let mut file = File::create("accuracy_results.txt").unwrap();

    let headline = format!(", {}, {}, {}", key[0], key[1], key[2]);

    let line1 = format!("\n{}, {} \u{00b1} {}, {} \u{00b1} {}, {} \u{00b1} {}", func[0], mean[0][0], std[0][0], mean[0][1], std[0][1], mean[0][2], std[0][2]);

    let line2 = format!("\n{}, {} \u{00b1} {}, {} \u{00b1} {}, {} \u{00b1} {}", func[1], mean[1][0], std[1][0], mean[1][1], std[1][1], mean[1][2], std[1][2]);

    let line3 = format!("\n{}, {} \u{00b1} {}, {} \u{00b1} {}, {} \u{00b1} {}", func[2], mean[2][0], std[2][0], mean[2][1], std[2][1], mean[2][2], std[2][2]);

    let line4 = format!("\n{}, {} \u{00b1} {}, {} \u{00b1} {}, {} \u{00b1} {}", func[3], mean[3][0], std[3][0], mean[3][1], std[3][1], mean[3][2], std[3][2]);

    let contents = vec![headline, line1, line2, line3, line4];

    println!("{:?}", &contents);

    for line in contents.iter(){
        file.write_all(line.as_bytes()).unwrap();
    }   

    return;

}

fn sum_N(x: &Vec<LWE>) -> Vec<LWE>{

    let mut y = x.clone();
    let n = (x.len() as f32).log2() as usize;

    for _ in 0..n{
        let mut tmp = vec![];
        
        for j in 0..( y.len() / 2 ){

            tmp.push( y[2*j].add_with_padding(&y[2*j+1]).unwrap() );
        }
        y = tmp;
    }
    
    let padd = y[0].encoder.nb_bit_padding - 1;

    y[0].remove_padding_inplace(padd).unwrap();

    return y;    
}


fn glycemic_variability_precentage(x: Vec<LWE>, bsk: &LWEBSK, ksk: &LWEKSK, precision: usize) -> Vec<LWE>{
    
    let len = x.len() as f64;
    let mut gvp = vec![];
    gvp.push(x[0].clone().sub_with_padding(&x[0]).unwrap());

    for i in 1..x.len(){
        gvp.push(x[i].clone().sub_with_padding(&x[i-1]).unwrap());
    }

    let enc = Encoder::new(4.9, 32.4, precision, 12+1).unwrap();
    for lwe in gvp.iter_mut(){
        *lwe = lwe.bootstrap_with_function(&bsk, |x| calc_l(x), &enc).unwrap();
    }
    
    gvp = sum_N(&gvp);

    for lwe in gvp.iter_mut(){
        *lwe = lwe.keyswitch(&ksk).unwrap();
    }

    let enc = Encoder::new(0., 10., precision, 3).unwrap();
    for lwe in gvp.iter_mut(){
        *lwe = lwe.bootstrap_with_function(&bsk, |x| F_GVP(100.*(x/(5.*len)) - 1.), &enc).unwrap();
    }

    return gvp;
}

fn mean_glucose(x: Vec<LWE>, bsk: &LWEBSK, precision: usize) -> Vec<LWE>{

    let len = x.len() as f64;
    let mut mean = sum_N(&x);

    let enc = Encoder::new(0., 10., precision, 3).unwrap();
    for lwe in mean.iter_mut(){
        *lwe = lwe.bootstrap_with_function(&bsk, |x| F_MG(x/len), &enc).unwrap();
    }

return mean;
}

fn positive_time_in_range(x: Vec<LWE>, bsk: &LWEBSK, ksk: &LWEKSK, precision: usize) -> Vec<LWE>{

    let len = x.len() as f64;

    let mut ptir = x;

    let enc = Encoder::new(0., 1., precision, 12+1).unwrap();
    for lwe in ptir.iter_mut(){
        *lwe = lwe.bootstrap_with_function(&bsk, |x| in_range(x, 180., 70.), &enc).unwrap();
    }

    ptir = sum_N(&ptir);
    
    for lwe in ptir.iter_mut(){
        *lwe = lwe.keyswitch(&ksk).unwrap();
    }

    let enc = Encoder::new(0., 10., precision, 3).unwrap();
    for lwe in ptir.iter_mut(){
        *lwe = lwe.bootstrap_with_function(&bsk, |x| F_PTIR( x/len ), &enc).unwrap();
    }

    return ptir;
}

fn hypo(x: Vec<LWE>, bsk: &LWEBSK, ksk: &LWEKSK, precision: usize) -> Vec<LWE>{

    let len = x.len() as f64;
    // HYPO 70
    let mut hypo54 = x.clone();

    let enc = Encoder::new(0., 1., precision, 12+1).unwrap();
    for lwe in hypo54.iter_mut(){
        *lwe = lwe.bootstrap_with_function(&bsk, |x| in_range(x, 54., 0.), &enc).unwrap();
    }

    hypo54 = sum_N(&hypo54);
    for lwe in hypo54.iter_mut(){
        *lwe = lwe.keyswitch(&ksk).unwrap();
    }

    let enc = Encoder::new(0., 5., precision, 4).unwrap();
    for lwe in hypo54.iter_mut(){
        *lwe = lwe.bootstrap_with_function(&bsk, |x| F_H54(x/len), &enc).unwrap();
    }

    // HYPO 70
    let mut hypo70 = x.clone();

    let enc = Encoder::new(0., 1., precision, 12+1).unwrap();
    for lwe in hypo70.iter_mut(){
        *lwe = lwe.bootstrap_with_function(&bsk, |x| in_range(x, 70., 0.), &enc).unwrap();
    }

    hypo70 = sum_N(&hypo70);
    for lwe in hypo70.iter_mut(){
        *lwe = lwe.keyswitch(&ksk).unwrap();
    }

    let enc = Encoder::new(0., 5., precision, 4).unwrap();
    for lwe in hypo70.iter_mut(){
        *lwe = lwe.bootstrap_with_function(&bsk, |x| F_H70(x/len), &enc).unwrap();
    }

    let mut hypo = vec![];
    for (lwe_54, lwe_70) in hypo54.iter().zip(hypo70.iter()){
        hypo.push( lwe_54.add_with_padding(&lwe_70).unwrap() );
    }

    return hypo;

}

fn plain_gvp(x: Vec<f64>) -> f64{
    
    let len = x.len() as f64;
    let mut gvp = vec![0.];
    for i in 1..x.len(){
        gvp.push(x[i]-x[i-1]);
    }

    for val in gvp.iter_mut(){
        *val = calc_l(*val);
    }

    let mut summed = 0.;
    for d in gvp.iter() {
        summed += d/len;
    }

    return F_GVP(100.*(summed/(5.)) -1.);
}

fn plain_mean(x: Vec<f64>) -> f64{

    let mean = x.clone();
    let len = x.len() as f64;

    let mut summed = 0.;
    for d in mean.iter() {
        summed += d/len;
    }

    return F_MG( summed );

}

fn plain_ptir(x: Vec<f64>) -> f64{

    let len = x.len() as f64;
    let mut ptir = x;
    
    for val in ptir.iter_mut(){
        *val = in_range(*val, 180., 70.);
    }

    let mut summed = 0.;
    for d in ptir.iter() {
        summed += d/len;
    }

    return F_PTIR( summed );

}

fn plain_hypo(x: Vec<f64>) -> f64{

    let len = x.len() as f64;
    let mut h70 = x.clone();
    let mut h54 = x.clone();

    for val in h70.iter_mut(){
        *val = in_range(*val, 70., 0.);
    }

    for val in h54.iter_mut(){
        *val = in_range(*val, 54., 0.);
    }

    let mut sum70 = 0.;
    for d in h70.iter() {
        sum70 += d/len;
    }

    let mut sum54 = 0.;
    for d in h54.iter() {
        sum54 += d/len;
    }

    return F_H70( sum70 ) + F_H54( sum54 );


}

fn calculate_accuracy() -> (Vec<Vec<f64>>, Vec<Vec<f64>>, Vec<Vec<f64>>, Vec<Vec<f64>>){
    
    let prec = vec![4, 5, 6];

    let lwe_dim = 750;//512, 1024, 2048];
    let lwe_noise = -29;//-19, -40, -62];
    
    let rlwe_dim = [1024, 2048, 4096];
    let rlwe_noise = -62; //-19, -40, -62];

    let base_log = 6;
    let lvl = 6;

    let mut data = vec![48., 50., 51., 56.];

    let mut E_gvp = vec![];
    let mut E_mg = vec![];
    let mut E_ptir = vec![];
    let mut E_hypo = vec![];


    for (dim, precision) in rlwe_dim.iter().zip(prec.iter()){

        let mut err_gvp = vec![];
        let mut err_mg = vec![];
        let mut err_ptir = vec![];
        let mut err_hypo = vec![];

        let lwe_params: LWEParams = LWEParams::new(lwe_dim, lwe_noise);
        let rlwe_params: RLWEParams = RLWEParams{polynomial_size: *dim, dimension: 1, log2_std_dev: rlwe_noise};
        

        let sk = LWESecretKey::new(&lwe_params);
        let sk_rlwe = RLWESecretKey::new(&rlwe_params);
        let sk_out = sk_rlwe.to_lwe_secret_key();
        
        let ksk = LWEKSK::new(&sk_out, &sk, base_log, lvl);
        let bsk = LWEBSK::new(&sk, &sk_rlwe, base_log, lvl);

        println!("{}", *dim);

        for i in 0..10{
            println!("{}", i);

            

            /*
            let mut summed = 0.;
            for d in data.iter() {
                summed += d;
            }
            println!("{}", summed);
            */
            
            let enc_gvp = Encoder::new(0., 400., 11, 2).unwrap();
            let enc_mean = Encoder::new(0., 400., 11, 12+1).unwrap();
            let enc_tir = Encoder::new(0., 400., 11, 1).unwrap();

            let mut enc_data_gpv = vec![];
            let mut enc_data_mean = vec![];
            let mut enc_data_tir = vec![];

            for d in data.iter(){
                enc_data_gpv.push( LWE::encode_encrypt(&sk, *d, &enc_gvp).unwrap() );
                enc_data_mean.push( LWE::encode_encrypt(&sk, *d, &enc_mean).unwrap() );
                enc_data_tir.push( LWE::encode_encrypt(&sk, *d, &enc_tir).unwrap() );
            }

            let gvp = glycemic_variability_precentage(enc_data_gpv, &bsk, &ksk, precision.clone());
            let mg = mean_glucose(enc_data_mean, &bsk, precision.clone());
            let ptir = positive_time_in_range(enc_data_tir.clone(), &bsk, &ksk, precision.clone());
            let hyp = hypo(enc_data_tir, &bsk, &ksk, precision.clone());
            
            let p_gvp = plain_gvp( data.clone() );
            let p_mg = plain_mean( data.clone() );
            let p_ptir = plain_ptir( data.clone() );
            let p_hypo = plain_hypo( data.clone() );

            data = data.into_iter().map(|x| x + 10.).collect();

            err_gvp.push( p_gvp - gvp[0].decrypt_decode(&sk_out).unwrap() );
            err_mg.push( p_mg - gvp[0].decrypt_decode(&sk_out).unwrap() );
            err_ptir.push( p_ptir - gvp[0].decrypt_decode(&sk_out).unwrap() );
            err_hypo.push( p_hypo - gvp[0].decrypt_decode(&sk_out).unwrap() );

            /*
            println!("GVP: plain = {}, enc = {}", plain_gvp( data.clone() ), gvp[0].decrypt_decode(&sk_out).unwrap()  );
            println!("MG: plain = {}, enc = {}", plain_mean( data.clone() ), mg[0].decrypt_decode(&sk_out).unwrap() );
            println!("TIR: plain = {}, enc = {}", plain_ptir( data.clone() ), ptir[0].decrypt_decode(&sk_out).unwrap() );
            println!("HG: plain = {}, enc = {}", plain_hypo( data.clone() ), hyp[0].decrypt_decode(&sk_out).unwrap() );
            */
        }

    E_gvp.push( err_gvp );
    E_mg.push( err_mg );
    E_ptir.push( err_ptir );
    E_hypo.push( err_hypo );
    
    }

    return (E_gvp, E_mg, E_ptir, E_hypo);

}

fn mean_std(x: Vec<Vec<f64>>) -> (Vec<f64>, Vec<f64>) {

    let len_0 = x.len();
    let len_1 = x[1].len();

    let mut means = vec![];
    let mut stds = vec![];

    for i in 0..len_0{

        let mean = x[i].iter().sum::<f64>();
        let mut std = 0.;

        for j in 0..len_1{
            std += (x[i][j] - mean).powf(2.);
        }

        means.push( mean );
        stds.push( std.powf(0.5) );
    }
    
    return (means, stds);
}

fn main() -> (){

    let mut means = vec![];
    let mut stds = vec![];

    let (gvp, mg, ptir, hypo) = calculate_accuracy();


    let (mean, std) = mean_std(gvp);
    means.push(mean);
    stds.push(std);

    let (mean, std) = mean_std(mg);
    means.push(mean);
    stds.push(std);

    let (mean, std) = mean_std(ptir);
    means.push(mean);
    stds.push(std);

    let (mean, std) = mean_std(hypo);
    means.push(mean);
    stds.push(std);

    let functions = vec!["Glycemic Variability", "Mean Glucose", "Positive TIR", "Hypo Glycemic"];

    write_res(functions, vec![1024, 2048, 4096], means, stds);
}


