use watt_pm::config::PackageType;

use super::init;

pub fn execute(path: &str, ty: Option<PackageType>) {
	// TODO: Handle I/O errors. (`unwrap()`)
	std::fs::create_dir(path).unwrap();

	std::env::set_current_dir(path).unwrap();

	init::execute(ty);
}
