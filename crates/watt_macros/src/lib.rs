/// Panics with the error
#[macro_export]
macro_rules! bail {
    ($report:expr) => {{
        let report: miette::Report = $report.into();
        panic!("{report:?}");
    }};
}

/// Panics with bug error
#[macro_export]
macro_rules! bug {
    ($text:expr) => {{
        panic!("{:?}", miette::miette!($text));
    }};
}
