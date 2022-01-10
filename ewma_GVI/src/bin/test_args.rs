use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("args.len(): {}", args.len());

    let base_log: u16 = args[1].parse().unwrap();
    let level: u16 = args[2].parse().unwrap();

    println!("base_log: {}", base_log);
    println!("level: {}", level);
}