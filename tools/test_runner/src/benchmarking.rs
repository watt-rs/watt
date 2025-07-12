use std::process::Stdio;

/// Benchmark options
pub struct BenchmarkOptions {
    sample_count: usize,
}

/// Default options
impl Default for BenchmarkOptions {
    fn default() -> Self {
        Self { sample_count: 200 }
    }
}

/// An implementation to change options
impl BenchmarkOptions {
    pub fn sample_count(mut self, count: usize) -> Self {
        self.sample_count = count;
        self
    }
}

/// Running benchmark with given compiler, target file and options.
pub fn run_benchmark_on(compiler_path: &str, filepath: &str, options: &BenchmarkOptions) {
    let mut mean_time: u128 = 0;

    for _ in 0..options.sample_count {
        // Create command instance and give it a file path
        let mut command = std::process::Command::new(compiler_path);
        let command = command.arg(filepath).stdout(Stdio::null());

        // Current time
        let start_time = std::time::Instant::now();

        // Run
        command.status().unwrap();

        // Time difference
        let diff = start_time.elapsed().as_nanos();

        // Calculating time midpoint: mean_time = (mean_time + diff) / 2
        mean_time = mean_time.midpoint(diff);
    }

    // 1 ms = 1_000_000 ns.
    println!(
        "[BENCH] {filepath}: {:.2} ms",
        (mean_time as f64) / 1_000_000.0
    );
}
