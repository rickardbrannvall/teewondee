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

fn main() -> Result<(), CryptoAPIError> {

    let upper = 400.;
    
    let key_path = "keys/80_1024_1";
    let data_path = "data/CGM_p77_24h";

    let mut base_log: usize = 6;
    let mut level: usize = 4; 
    let args: Vec<String> = env::args().collect();
    if args.len() == 3 {
        base_log =  args[1].parse().unwrap();
        level = args[2].parse().unwrap();
    }
    println!("base_log {}", base_log);
    println!("level {}", level);    
    let key_path = format!("{}_{}_{}",key_path,base_log,level);
    println!("key path {}", key_path);    
    let sk0_LWE_path = format!("{}/sk0_LWE.json",key_path);
    let sk0 = LWESecretKey::load(&sk0_LWE_path).unwrap();    

    let enc = Encoder::new(0., upper, 3, 2).unwrap();   
            
    let csvfile = format!("{}.csv",data_path);
    let data = fs::read_to_string(csvfile).expect("Unable to read file");
    
    let mut reader = ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b',')
        .from_reader(data.as_bytes());
    
    let mut pt: Vec<f64> = Vec::new();
    
    for record in reader.deserialize() {
        let record: Record = record.unwrap();
        pt.push(record.value); 
    }
    
    let ct = VectorLWE::encode_encrypt(&sk0, &pt, &enc)?; 
    
    let encfile = format!("{}_{}_{}.enc",data_path,base_log,level);
    println!("write {}",encfile);
    
    ct.save(&encfile).unwrap();    

    Ok(())
}
