use std::process::Stdio;

/// Опции бенчмарка
pub struct BenchmarkOptions {
    sample_count: usize,
}

/// Опции бенчмарка применяемые по умолчанию
impl Default for BenchmarkOptions {
    fn default() -> Self {
        Self { sample_count: 200 }
    }
}

/// Методы для настройки опций бенчмарка
impl BenchmarkOptions {
    pub fn sample_count(mut self, count: usize) -> Self {
        self.sample_count = count;
        self
    }
}

/// Запуск бенчмарка с необходимым компилятором, путём к программе и опциями
pub fn run_benchmark_on(compiler_path: &str, filepath: &str, options: &BenchmarkOptions) {
    let mut mean_time: u128 = 0;

    for _ in 0..options.sample_count {
        // Создаём команду, добавляем туда аргумент пути к программе и перенаправляем stdout в никуда
        let mut command = std::process::Command::new(compiler_path);
        let command = command.arg(filepath).stdout(Stdio::null());

        // Засекаем время
        let start_time = std::time::Instant::now();

        // Запускаем
        command.status().unwrap();

        // Вычисляем разницу во времени
        let diff = start_time.elapsed().as_nanos();

        // Вычисляем среднее время: mean_time = (mean_time + diff) / 2
        mean_time = mean_time.midpoint(diff);
    }

    // 1 ms = 1_000_000 ns.
    println!("[BENCH] {filepath}: {:.2} ms", (mean_time as f64) / 1_000_000.0);
}