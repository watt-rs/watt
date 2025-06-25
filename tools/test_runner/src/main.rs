use crate::benchmarking::BenchmarkOptions;

pub mod testing;
pub mod benchmarking;

const AVAILABLE_MODES: &[&str] = &["test", "bench"];

fn main() {
    let mut arguments = std::env::args();
    let program_name = arguments.next().unwrap();
	// Первым аргументом программы является путь к корню исходного кода. (где хранятся исходники).
    let mode = match arguments.next() {
        Some(mode) => mode,
        None => {
            eprintln!("No mode selected! Run: `{program_name} help`");
            std::process::exit(1);
        }
    };

    let working_directory = match arguments.next() {
        Some(dir) => dir,
        None => {
            eprintln!("Please specify working Watt directory!");
            std::process::exit(1);
        }
    };

	// Переходим к рабочей директории.
    std::env::set_current_dir(&working_directory).unwrap();

    let compiler_path = working_directory.clone() + "/target/release/Watt";

    if !std::fs::exists(&compiler_path).unwrap_or(false) {
        eprintln!(
            "The compiler seem not built yet. Build it first by running `cargo b --release`!"
        );
        std::process::exit(1);
    }

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

            // Запускаем тесты.
            let stats = testing::run_tests(&compiler_path, &tests_table);

            println!("test results: {} ran: {} ok, {} fail", stats.ran, stats.ok, stats.fail);
        }
        "bench" => {
            let file = match arguments.next() {
                Some(file) => file,
                None => {
                    eprintln!("No .wt file specified.");
                    std::process::exit(1);
                }
            };

            benchmarking::run_benchmark_on(&compiler_path, &file, &BenchmarkOptions::default());
        }
        _ => {
            eprintln!("Invalid mode: {mode:?}! Run: `{program_name} help`");
            std::process::exit(1);
        }
    }
}
