
use bindgen::CodegenConfig;

use std::fs::File;
use std::path::{Path, PathBuf};
use std::ffi::OsStr;

use clap::{Arg, App, SubCommand};


const PIP_ATARI_PY_TAR_URL: &'static str = "https://files.pythonhosted.org/packages/43/dd/2721f34a89dc520d2e09363fd23d110a33bbab2399e50fdced6eb2ed2157/atari-py-0.2.6.tar.gz";
const ATARI_PY_TAR_FILENAME: &'static str = "atari-py-0.2.6.tar.gz";

const XTASK_PREFIX: &'static str = "\x1B[1m\x1B[32m       xtask\x1B[0m ";
const ERROR_PREFIX: &'static str = "\x1B[1m\x1B[31merror\x1B[37m:\x1B[0m ";

fn main() {
	let mut app = App::new("ale-xtask")
		.version("0.1.0")
		.about("Build runner for the ale project")
		.author("Callum Tolley")
		.subcommand(SubCommand::with_name("gen-bindings")
			.about("Generate Arcade Learning Environment bindings"))
		.subcommand(SubCommand::with_name("download-roms")
			.about("Download builtin Atari ROMs, and place in the roms/ folder"))
		.subcommand(SubCommand::with_name("clean")
			.about("Remove the target directories")
			.arg(Arg::with_name("all")
				.long("all")
				.help("Remove the xtask target directory")));

	let matches = app.clone().get_matches();

	if let Some(_) = matches.subcommand_matches("gen-bindings") {
		eprintln!("{}gen-bindings", XTASK_PREFIX);
		run_bindgen();

	} else if let Some(_) = matches.subcommand_matches("download-roms") {
		eprintln!("{}download-roms", XTASK_PREFIX);
		run_download_roms();

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
	eprintln!("{}run bindgen", XTASK_PREFIX);
	let bindings = match bindgen::builder()
		.clang_arg(format!("-I{}", project_root().join("ale-sys").join("ale").join("src").display()))
		.clang_arg(format!("-I{}", project_root().join("ale-sys").join("ale").join("ale_py").display()))
		.clang_args(&["-x", "c++"])
		.clang_arg("-std=c++14")
		.enable_cxx_namespaces()
		.header(format!("{}", project_root().join("ale-sys").join("wrapper.h").display()))
		.whitelist_function("ALE_new")
		.whitelist_function("ALE_del")
		.whitelist_function("getString")
		.whitelist_function("getInt")
		.whitelist_function("getBool")
		.whitelist_function("getFloat")
		.whitelist_function("setString")
		.whitelist_function("setInt")
		.whitelist_function("setBool")
		.whitelist_function("setFloat")
		.whitelist_function("loadROM")
		.whitelist_function("act")
		.whitelist_function("game_over")
		.whitelist_function("reset_game")
		.whitelist_function("getAvailableModes")
		.whitelist_function("getAvailableModesSize")
		.whitelist_function("setMode")
		.whitelist_function("getAvailableDifficulties")
		.whitelist_function("getAvailableDifficultiesSize")
		.whitelist_function("setDifficulty")
		.whitelist_function("getLegalActionSet")
		.whitelist_function("getLegalActionSize")
		.whitelist_function("getMinimalActionSet")
		.whitelist_function("getMinimalActionSize")
		.whitelist_function("getFrameNumber")
		.whitelist_function("lives")
		.whitelist_function("getEpisodeFrameNumber")
		.whitelist_function("getScreen")
		.whitelist_function("getRAM")
		.whitelist_function("getRAMSize")
		.whitelist_function("getScreenWidth")
		.whitelist_function("getScreenHeight")
		.whitelist_function("getScreenRGB")
		.whitelist_function("getScreenGrayscale")
		.whitelist_function("saveState")
		.whitelist_function("loadState")
		.whitelist_function("cloneState")
		.whitelist_function("restoreState")
		.whitelist_function("cloneSystemState")
		.whitelist_function("restoreSystemState")
		.whitelist_function("deleteState")
		.whitelist_function("saveScreenPNG")
		.whitelist_function("encodeState")
		.whitelist_function("encodeStateLen")
		.whitelist_function("decodeState")
		.whitelist_function("setLoggerMode")
		.opaque_type(".*")
		.with_codegen_config(CodegenConfig::FUNCTIONS | CodegenConfig::TYPES)
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

fn run_download_roms() {
	let dir = tempdir::TempDir::new("ale-xtask").expect("failed to generate temp directory");
	let tar_path = dir.path().join(ATARI_PY_TAR_FILENAME);
	let extract_dir = dir.path().join("extract");

	run_download(PIP_ATARI_PY_TAR_URL, &tar_path);
	run_extract(&tar_path, &extract_dir);

	let roms_dir = project_root().join("roms");
	std::fs::create_dir_all(&roms_dir).expect("failed to create roms dir");
	for rom in std::fs::read_dir(extract_dir.join("atari-py-0.2.6").join("atari_py").join("atari_roms")).expect("failed to read dir") {
		let rom = rom.expect("failed to read dir");
		eprintln!("{}copy {}", XTASK_PREFIX, rom.path().file_name().unwrap_or(OsStr::new("")).to_string_lossy());
		std::fs::copy(rom.path(), roms_dir.join(rom.path().file_name().unwrap()))
			.expect("failed to copy file");
	}
}

fn run_download(url: &str, dst: &Path) {
	eprintln!("{}download {}", XTASK_PREFIX, url);
	let mut out = File::create(&dst).expect("failed to create dst file");
	let mut res = match reqwest::blocking::get(url) {
		Err(e) => {
			eprintln!("{}failed to download {}: {}", ERROR_PREFIX, url, e);
			::std::process::exit(1);
		},
		Ok(r) => r,
	};

	if !res.status().is_success() {
		eprintln!("{}failed to download {}: status is {:?}", ERROR_PREFIX, url, res.status());
		::std::process::exit(1);
	}

	if let Err(e) = res.copy_to(&mut out) {
		eprintln!("{}failed to download {}: {}", ERROR_PREFIX, url, e);
		::std::process::exit(1);
	}
}

fn run_extract(tar_path: &Path, extract_dir: &Path) {
	eprintln!("{}extract {} to {}", XTASK_PREFIX, tar_path.file_name().unwrap_or(OsStr::new("")).to_string_lossy(), extract_dir.display());
	std::fs::create_dir_all(&extract_dir).expect("failed to create extract dir");

	let tar_gz = File::open(tar_path).expect("failed to open tar.gz");
	let tar = flate2::read::GzDecoder::new(tar_gz);
	let mut archive = tar::Archive::new(tar);
	archive.unpack(extract_dir).expect("failed to extract tar.gz");
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
