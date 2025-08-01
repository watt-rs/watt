use std::{env, fs::File};
use std::io::Write;

fn main() {
    let target = env::var("TARGET").expect("Environment variable TARGET must exist.");
    let out_dir = env::var("OUT_DIR").expect("Environment variable OUT_DIR must exist.");
    
    let mut file = File::create(out_dir + "/target.txt").expect("Failed to create target.txt");
    write!(&mut file, "{}", target).expect("Failed to write target into target.txt");
}
