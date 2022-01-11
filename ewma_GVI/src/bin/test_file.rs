//use std::error::Error;
use csv::ReaderBuilder;
use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
struct Record {
    _time: String,
    value: f64,
}


fn main() -> Result<(), csv::Error> {
//    let csv = "time,value
//1948,179.
//1967,182.";

    //let mut reader = csv::Reader::from_reader(csv.as_bytes());

    let data = fs::read_to_string("../data/CGM_p77.csv").expect("Unable to read file");
    let mut reader = ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b',')
        .from_reader(data.as_bytes());
    
    let mut data: Vec<f64> = Vec::new();
    
    for record in reader.deserialize() {
        let record: Record = record.unwrap();
        data.push(record.value); 
    }

    Ok(())
}