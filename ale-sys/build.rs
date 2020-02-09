
use std::io;
use std::path::{Path, PathBuf};
use std::fs::{self, DirEntry};

#[cfg(windows)]
fn is_win() -> bool {
	true
}

#[cfg(not(windows))]
fn is_win() -> bool {
	false
}

fn visit_dirs<F: FnMut(&DirEntry)>(dir: &Path, cb: &mut F) -> io::Result<()> {
	if dir.is_dir() {
		for entry in fs::read_dir(dir)? {
			let entry = entry?;
			let path = entry.path();
			if path.is_dir() {
				visit_dirs(&path, cb)?;
			} else {
				cb(&entry);
			}
		}
	}
	Ok(())
}

fn main() {
	let debug = std::env::var("DEBUG").expect("DEBUG env var required")
		.parse::<bool>().expect("DEBUG not set to valid bool");
	let profile = match std::env::var("OPT_LEVEL").unwrap().as_str() {
		"0" => "Debug",
		"1" | "2" | "3" => if debug { "RelWithDebInfo" } else { "Release" },
		"s" | "z" => "MinSizeRel",
		lvl => panic!("Unknown OPT_LEVEL: {}", lvl),
	};

	let ale_dir = project_root().join("ale");
	let mut config = cmake::Config::new(&ale_dir);
	config
		.define("USE_SDL", "OFF")
		.define("USE_RLGLUE", "OFF")
		.define("BUILD_EXAMPLES", "OFF")
		.define("BUILD_CPP_LIB", "OFF")
		.define("BUILD_CLI", "OFF")
		.define("BUILD_C_LIB", "ON")
		.profile(profile)
		.build_target("ale-c-lib");

	if is_win() {
		config.cflag("-DWIN32=1").cxxflag("-DWIN32=1");
	}

	let dst = config.build();

	let mut ale_files = vec![];
	visit_dirs(&ale_dir, &mut |de| ale_files.push(de.path())).expect(&format!("visit_dirs failed in {}", ale_dir.display()));

	for path in ale_files {
		println!("cargo:rerun-if-changed={}", path.display());
	}
	println!("cargo:rerun-if-changed=build.rs");
	println!("cargo:rustc-link-search=native={}", dst.join("build").join(profile).display());
	println!("cargo:rustc-link-lib=static=ale_c");
}

fn project_root() -> PathBuf {
	Path::new(&env!("CARGO_MANIFEST_DIR")).to_path_buf()
}
