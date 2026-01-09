use watt_pm::{config::PackageType, generate};

pub fn execute(path: &str, pkg_ty: Option<PackageType>) {
	std::fs::create_dir(path).unwrap();

    let pkg_ty = pkg_ty.unwrap_or(PackageType::App);

    generate::gen_project(path.into(), pkg_ty);
}
