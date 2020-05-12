
use std::io;
use std::path::{Path, PathBuf};
use std::fs::{self, DirEntry};
use std::ffi::OsStr;

#[cfg(windows)]
fn is_win() -> bool {
	true
}

#[cfg(not(windows))]
fn is_win() -> bool {
	false
}

fn visit_dirs<F: FnMut(&DirEntry), G: Fn(&DirEntry) -> bool>(dir: &Path, cb: &mut F, filter: &G) -> io::Result<()> {
	if dir.is_dir() {
		for entry in fs::read_dir(dir)? {
			let entry = entry?;
			let path = entry.path();
			if !filter(&entry) {
				continue;
			}
			if path.is_dir() {
				visit_dirs(&path, cb, filter)?;
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

	let temp_dir = tempdir::TempDir::new("ale-sys-build").expect("failed to create temp dir");
	println!("temp_dir={}", temp_dir.path().display());
	let lib_dir = out_dir().join("build").join("lib");

	let cwd = std::env::current_dir().expect("failed to get current dir");
	std::env::set_current_dir(&temp_dir).expect("failed to set current dir");

	let ale_dir = project_root().join("ale");
	let mut config = cmake::Config::new(&ale_dir);
	config
		.define("USE_SDL", "OFF")
		.define("USE_RLGLUE", "OFF")
		.define("BUILD_EXAMPLES", "OFF")
		.define("BUILD_CPP_LIB", "OFF")
		.define("BUILD_CLI", "OFF")
		.define("BUILD_C_LIB", "ON")
		.define(format!("CMAKE_ARCHIVE_OUTPUT_DIRECTORY_{}", &profile.to_uppercase()), &lib_dir)
		.profile(profile)
		.build_target("ale-c-lib-static");

	if is_win() {
		config.cflag("-DWIN32=1").cxxflag("-DWIN32=1");
	}

	let dst = config.build();
	std::env::set_current_dir(&cwd).expect("failed to set current dir");
	println!("dst={}", dst.display());

	let ignore_files: &[&OsStr] = &["build".as_ref(), ".git".as_ref()];
	let mut ale_files = vec![];
	visit_dirs(&ale_dir, &mut |de| ale_files.push(de.path()), &|de| de.path().file_name().map(|n| !ignore_files.contains(&n)).unwrap_or(true)).expect(&format!("visit_dirs failed in {}", ale_dir.display()));

	for path in ale_files {
		println!("cargo:rerun-if-changed={}", path.display());
	}
	println!("cargo:rerun-if-changed=build.rs");
	if !is_win() {
		println!("cargo:rustc-flags=-l dylib=stdc++");
	}
	println!("cargo:rustc-link-search=native={}", lib_dir.display());
	println!("cargo:rustc-link-lib=static=ale_c_static");
}

fn project_root() -> PathBuf {
	Path::new(&env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn out_dir() -> PathBuf {
	Path::new(&std::env::var_os("OUT_DIR").expect("OUT_DIR not defined")).to_path_buf()
}
