mod lib;
use lib::*;

use concrete::*;

fn write_res(func: Vec<String>, key_size: Vec<usize>, mean: Vec<f64>, std: Vec<f64>) -> (){

    let mut file = File::create("accuracy_results.txt").unwrap();

    let mut contents: String;

    for key in key_size.iter(){
        
    }

    for (i, f) in func.iter().enumerate(){
        contents = format!("{}\n\n{}\n\n", contents)
        for (key, m, s) in (key_size.iter().zip(mean.iter().zip(std.iter()))){

            let line = format!("Key {}: Error = {} \u{00b1} {}", key, m, s);

            file.write(b)
        }
    }
    return;

}
