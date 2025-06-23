fn main() {
    let compiler_path = match std::env::args().skip(1).next() {
        Some(path) => path,
        None => {
            eprintln!("Select path to Watt interpreter!");
            std::process::exit(1);
        }
    };

    if !std::fs::exists(compiler_path).unwrap_or(false) {
        eprintln!("The compiler seems not built yet. Build it first by running `cargo b --release`!");
        std::process::exit(1);
    }



    println!("Hello, world!");
}
