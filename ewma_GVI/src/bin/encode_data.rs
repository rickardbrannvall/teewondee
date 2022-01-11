#![allow(non_snake_case)]
use concrete::*;
use std::env;

use csv::ReaderBuilder;
use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
struct Record {
    _time: String,
    value: f64,
}

fn main() -> Result<(), CryptoAPIError> { // encode_data keys data padd prec high

    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        println!("format: encode keys data padd prec high");
        return Ok(());
    }
    
    let mut keys = "80_1024_1_5_3";
    let mut data = "CGM_p77_24h";
    let mut padd: usize = 2;
    let mut prec: usize = 3; 
    let mut high: usize = 400;
    if args.len() == 6 {
        keys = &args[1];
        data = &args[2];
        padd =  args[3].parse().unwrap();
        prec = args[4].parse().unwrap();
        high = args[5].parse().unwrap();
    }
    println!("assume: encode {} {} {} {} {}", keys, data, padd, prec, high);
           
    let sk0_LWE_path = format!("keys/{}/sk0_LWE.json",keys);
    let sk0 = LWESecretKey::load(&sk0_LWE_path).unwrap();    

    let enc = Encoder::new(0., high as f64, prec, padd).unwrap();   
            
    let csv_file = format!("data/{}.csv",data);
    let datafile = fs::read_to_string(csv_file).expect("Unable to read file");
    
    let mut reader = ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b',')
        .from_reader(datafile.as_bytes());
    
    let mut pt: Vec<f64> = Vec::new();
    
    for record in reader.deserialize() {
        let record: Record = record.unwrap();
        pt.push(record.value); 
    }
    
    let ct = VectorLWE::encode_encrypt(&sk0, &pt, &enc)?; 

    let path = format!("data/{}",keys);
    fs::create_dir_all(&path).unwrap();
    
    let encfile = format!("{}/{}_{}_{}_{}.enc",path,data,padd,prec,high);
    println!("write to: {}",encfile);
    
    ct.save(&encfile).unwrap();    

    Ok(())
}
