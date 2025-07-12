use crate::benchmarking::BenchmarkOptions;

pub mod benchmarking;
pub mod testing;

const AVAILABLE_MODES: &[&str] = &["test", "bench"];

fn main() {
    let mut arguments = std::env::args();
    let program_name = arguments.next().unwrap();

    // First argument is a subcommand.
    let mode = match arguments.next() {
        Some(mode) => mode,
        None => {
            eprintln!("No mode selected! Run: `{program_name} help`");
            std::process::exit(1);
        }
    };

    // The second argument is a working directory.
    let working_directory = match arguments.next() {
        Some(dir) => dir,
        None => {
            eprintln!("Please specify working Watt directory!");
            std::process::exit(1);
        }
    };

    // Switch to working directory.
    std::env::set_current_dir(&working_directory).unwrap();

    let compiler_path = working_directory.clone() + "/target/release/Watt";

    println!("Building Watt...");
    if let Err(e) = std::process::Command::new("cargo")
        .args(["b", "--release"])
        .spawn()
        .map(|mut ch| ch.wait())
    {
        eprintln!("Error occured when building Watt: {e:?}");

        std::process::exit(1);
    };

    match mode.as_str() {
        "help" => {
            eprintln!("Usage: {program_name} mode working_directory [FILES...]");
            eprintln!("Available modes are: {:?}", AVAILABLE_MODES);
            std::process::exit(1);
        }
        "test" => {
            let tests_path = working_directory.clone() + "/tests";

            // Собираем данные о тестах.
            let tests_table = match testing::build_verification_table(&tests_path) {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("Failed to build tests table. ({e:?})");
                    std::process::exit(1);
                }
            };

            println!("----- Running tests -----");

            // Run tests.
            let stats = testing::run_tests(&compiler_path, &working_directory, &tests_table);

            println!(
                "test results: {} ran: {color_green}{}{color_end} ok, {color_red}{}{color_end} fail",
                stats.ran,
                stats.ok,
                stats.fail,
                color_green = "\x1b[32;1m",
                color_red = "\x1b[31;1m",
                color_end = "\x1b[0m"
            );

            if stats.fail != 0 {
                std::process::exit(1);
            }
        }
        "bench" => {
            let file = match arguments.next() {
                Some(file) => file,
                None => {
                    eprintln!("No .wt file specified.");
                    std::process::exit(1);
                }
            };

            println!("----- Running benchmarks -----");

            benchmarking::run_benchmark_on(&compiler_path, &file, &BenchmarkOptions::default());
        }
        _ => {
            eprintln!("Invalid mode: {mode:?}! Run: `{program_name} help`");
            std::process::exit(1);
        }
    }
}
