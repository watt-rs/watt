use std::process::Stdio;

pub struct BenchmarkOptions {
    sample_count: usize,
}

impl Default for BenchmarkOptions {
    fn default() -> Self {
        Self { sample_count: 200 }
    }
}

impl BenchmarkOptions {
    pub fn sample_count(mut self, count: usize) -> Self {
        self.sample_count = count;
        self
    }
}

pub fn run_benchmark_on(compiler_path: &str, filepath: &str, options: &BenchmarkOptions) {
    let mut mean_time: u128 = 0;

    for _ in 0..options.sample_count {
        let mut command = std::process::Command::new(compiler_path);
        let command = command.arg(filepath).stdout(Stdio::null());

        let start_time = std::time::Instant::now();

        command.status().unwrap();

        let diff = start_time.elapsed().as_nanos();

        mean_time = mean_time.midpoint(diff);
    }

    // 1 ms = 1_000_000 ns.
    println!("[BENCH] {filepath}: {:.2} ms", (mean_time as f64) / 1_000_000.0);
}