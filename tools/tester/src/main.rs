use std::{collections::HashMap, path::PathBuf, process::ExitStatus};

type TestMap = HashMap<String, Option<String>>;

#[derive(Default)]
struct TesterStats {
    ran: usize,
    ok: usize,
    fail: usize
}

fn find_tests(on: &str) -> std::io::Result<Vec<PathBuf>> {
    Ok(std::fs::read_dir(on)?
        .map(|x| x.unwrap().path())
        .filter(|x| x.extension().map(|k| k.to_str().unwrap()) == Some("wt"))
        .collect())
}

fn build_verification_table(on: &str) -> std::io::Result<TestMap> {
    let tests = find_tests(on)?;
    let outputs_path = on.to_string() + "/output/";

    let mut table = HashMap::with_capacity(tests.len());

    for i in tests {
        let output_file: Option<String> = {
            let output_path = outputs_path.clone() + i.file_name().unwrap().to_str().unwrap() + ".stdout";

            if std::fs::exists(&output_path).unwrap_or(false) {
                Some(output_path)
            } else {
                None
            }
        };

        table.insert(i.to_str().unwrap().to_string(), output_file);
    }

    Ok(table)
}

fn run_tests(watt_path: &str, tests_table: &HashMap<String, Option<String>>) -> TesterStats {
    let mut stats = TesterStats::default();

    for (test_file, expected_content_file) in tests_table {
        let mut command = std::process::Command::new(watt_path);
        let command = command.arg(test_file);

        match command.output() {
            Ok(data) => {
                if !data.status.success() {
                    println!("[FAIL] {test_file}");
                    
                    stats.fail += 1;
                    stats.ran += 1;

                    continue;
                }

                if expected_content_file.is_none() {
                    println!("[NO OUTPUT - OK] {test_file}");

                    stats.ok += 1;
                } else {
                    let verify_data = std::fs::read(expected_content_file.as_ref().unwrap()).unwrap();

                    if verify_data == data.stdout {
                        println!("[OK] {test_file}");

                        stats.ok += 1;
                    } else {
                        println!("[FAIL] {test_file}");
                        println!("Expected:\n---");

                        print!("{}", str::from_utf8(&verify_data).unwrap());
                        println!("\n---");

                        println!("Got:\n---");

                        print!("{}", str::from_utf8(&data.stdout).unwrap());
                        println!("\n---");

                        stats.fail += 1;
                    }
                }
            },
            Err(err) => {
                println!("[ERR] {test_file}: {err:?}");

                stats.fail += 1;
            },
        };

        stats.ran += 1;
    }

    assert!(stats.ran == stats.ok + stats.fail);

    stats
}

fn main() {
    let working_directory = match std::env::args().skip(1).next() {
        Some(path) => path,
        None => {
            eprintln!("Could not determine a working directory!");
            std::process::exit(1);
        }
    };

    let compiler_path = working_directory.clone() + "/target/release/Watt";
    let tests_path = working_directory.clone() + "/tests";

    // println!("{:?}", compiler_path);
    // println!("{:?}", tests_path);

    if !std::fs::exists(&compiler_path).unwrap_or(false) {
        eprintln!(
            "The compiler seem not built yet. Build it first by running `cargo b --release`!"
        );
        std::process::exit(1);
    }

    let tests_table = match build_verification_table(&tests_path) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Failed to build tests table. ({e:?})");
            std::process::exit(1);
        }
    };

    std::env::set_current_dir(&working_directory).unwrap();

    let stats = run_tests(&compiler_path, &tests_table);

    println!("test results: {} ran: {} ok, {} fail", stats.ran, stats.ok, stats.fail);
}