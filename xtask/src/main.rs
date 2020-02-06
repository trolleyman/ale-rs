
use std::path::{Path, PathBuf};

use clap::{Arg, App, SubCommand};


const XTASK_PREFIX: &'static str = "\x1B[1m\x1B[32m       xtask\x1B[0m ";
const ERROR_PREFIX: &'static str = "\x1B[1m\x1B[31merror\x1B[37m:\x1B[0m ";

fn main() {
	let mut app = App::new("ale-xtask")
		.version("0.1.0")
		.about("Build runner for the ale project")
		.author("Callum Tolley")
		.subcommand(SubCommand::with_name("gen-bindings")
			.about("Generate Arcade Learning Environment bindings"))
		.subcommand(SubCommand::with_name("clean")
			.about("Remove the target directories")
			.arg(Arg::with_name("all")
				.long("all")
				.help("Remove the xtask target directory")));

	let matches = app.clone().get_matches();

	if let Some(_) = matches.subcommand_matches("gen-bindings") {
		eprintln!("{}gen-bindings", XTASK_PREFIX);
		run_bindgen();

	} else if let Some(matches) = matches.subcommand_matches("clean") {
		eprintln!("{}clean", XTASK_PREFIX);
		let mut rets = vec![
			run_rmdir(project_root().join("target"), false),
		];
		if matches.is_present("all") {
			rets.push(run_rmdir(project_root().join("xtask").join("target"), false));
		}
		if rets.iter().any(|r| r.is_err()) {
			std::process::exit(1);
		}
	} else {
		eprintln!("{}no subcommand specified", ERROR_PREFIX);
		app.print_help().expect("Failed to print help");
	}
}

fn run_bindgen() {
	eprintln!("{}generate bindings", XTASK_PREFIX);
	let bindings = match bindgen::builder()
		.clang_arg(format!("-I{}", project_root().join("ale-sys").join("ale").join("src").display()))
		.clang_arg(format!("-I{}", project_root().join("ale-sys").join("ale").join("ale_py").display()))
		.header("wrapper.h")
		.generate() {
			Ok(b) => b,
			Err(e) => {
				eprintln!("{}failed to generate bindings: {:?}", ERROR_PREFIX, e);
				std::process::exit(1);
			}
		};
	eprintln!("{}write bindings", XTASK_PREFIX);
	if let Err(e) = bindings.write_to_file(project_root().join("ale-sys").join("src").join("bindings.rs")) {
		eprintln!("{}failed to write bindings: {:?}", ERROR_PREFIX, e);
		std::process::exit(1);
	}
}

fn run_rmdir(dir: impl AsRef<Path>, error_fail: bool) -> Result<(), ()> {
	let dir = dir.as_ref();
	eprintln!("{}delete directory {}", XTASK_PREFIX, dir.display());
	if let Err(e) = fs_extra::dir::remove(dir) {
		eprintln!("{}failed to delete directory: {}", ERROR_PREFIX, e);
		if error_fail {
			std::process::exit(1);
		}
		Err(())
	} else {
		Ok(())
	}
}

fn project_root() -> PathBuf {
	Path::new(&env!("CARGO_MANIFEST_DIR"))
		.ancestors()
		.nth(1)
		.unwrap()
		.to_path_buf()
}
