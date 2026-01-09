/// Prints error, and then
/// exits proccess using `std::process::exit(1)`.
#[macro_export]
macro_rules! bail {
    ($report:expr) => {{
        let report: miette::Report = $report.into();
        eprintln!("{report:?}");
        std::process::exit(1);
    }};
}

/// Prints warning
#[macro_export]
macro_rules! warn {
    ($pkg:expr, $report:expr) => {{
        let report: miette::Report = $report.into();
        let report_code = report.code().unwrap().to_string();
        if !$pkg.draft.lints.disabled.contains(&report_code) {
            eprintln!("{report:?}");
        }
    }};
}
