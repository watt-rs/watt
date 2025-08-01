use std::io::{self, Write};

use clap::Arg;
use zip::write::SimpleFileOptions;

// This will be a default target if `--target` argument is not specified.
const DEFAULT_TARGET: &str = include_str!(concat!(env!("OUT_DIR"), "/target.txt"));

// Build Watt in release mode for gived `target_triplet`.
fn build_watt(watt_dir: &str, target_triplet: &str) -> io::Result<()> {
    std::env::set_current_dir(watt_dir)?;

    let mut cargo_build = std::process::Command::new("cargo");
    let cmd = cargo_build.arg("build").arg("--release").arg("--target").arg(target_triplet);

    let mut chld = cmd.spawn()?;
    chld.wait()?;

    Ok(())
}

// Pack Watt binary and its stdlib into `.zip` file.
fn pack_watt(watt_dir: &str, target_triplet: &str) -> io::Result<()> {
    let zipfile_name = "watt-".to_owned() + target_triplet + ".zip";
    // Open zip file in CWR (create-write-read) mode.
    let file = std::fs::OpenOptions::new().write(true).read(true).create(true).open(zipfile_name)?;

    let watt_path_len = watt_dir.len();
    
    let stdlib_dir = watt_dir.to_owned() + "/libs";
    let watt_path = watt_dir.to_owned() + "/target/" + target_triplet + "/release/Watt";

    println!("{watt_path:?}");

    // Create zip writer.
    let mut zip = zip::ZipWriter::new(file);

    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

    // Firstly, we write the binary itself.
    zip.start_file("watt", options.unix_permissions(0o755))?;

    let watt_data = std::fs::read(watt_path)?;
    zip.write(&watt_data)?;

    // Now, we're going to write the entire stdlib into zip file.
    for i in walkdir::WalkDir::new(stdlib_dir) {
        // Get the full path of the directory.
        let path = i.as_ref().unwrap().path().to_str().unwrap();

        // And cut the trailing path out of it (+ trailing slash).
        let normalized_path = &path[watt_path_len + 1..];

        let ftype = i.as_ref().unwrap().file_type();

        if ftype.is_dir() {
            zip.add_directory(normalized_path, SimpleFileOptions::default())?;

            println!("Creating: {normalized_path:?}");
        } else if ftype.is_file() {
            zip.start_file(normalized_path, options)?;
            zip.write(&std::fs::read(normalized_path)?)?;
            
            println!("Writing: {normalized_path:?}");
        }
    }

    zip.finish()?;

    Ok(())
}

// The function that controls the whole process
fn worker(watt_dir: &str, target_triplet: &str) {
    println!("Building Watt for target `{target_triplet}`...");

    let task = build_watt(watt_dir, target_triplet);

    if let Err(e) = task {
        eprintln!("Error occured when compiling Watt: {e:?}");

        std::process::exit(1);
    }

    let zipfile_name = "watt-".to_owned() + target_triplet + ".zip";
    
    println!("Packing Watt and stdlib into `{zipfile_name}`...");

    let task = pack_watt(watt_dir, target_triplet);

    if let Err(e) = task {
        eprintln!("Failed to pack Watt and its stdlib into zip file. (Error: {e:?})");

        std::process::exit(1);
    }
}

// The main function.
// WARNING: The `current_dir` argument is set by `./w` tool.
// Don't try running it manually.
fn main() {
    let parser = clap::Command::new("dist-packer")
        .author("Watt developers")
        .about("The Watt distribution packing utility.")
        .arg(
            Arg::new("current_dir")
                .required(true)
        )
        .arg(
            Arg::new("target")
                .long("target")
                .default_value(DEFAULT_TARGET),
        );
    
    let matches = parser.get_matches();

    let watt_dir: &String = matches.get_one("current_dir").unwrap();
    let target_triplet: &String = matches.get_one("target").unwrap();

    worker(watt_dir, target_triplet);
}
