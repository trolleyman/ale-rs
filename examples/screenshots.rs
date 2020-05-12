
use std::path::Path;
use std::ffi::CString;

use rand::prelude::*;

use ale::{Ale, BundledRom};

fn main() {
	let screenshots_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("screenshots").join("breakout");
	std::fs::create_dir_all(&screenshots_dir).expect("failed to create screenshots dir");

	let mut ale = Ale::new();
	ale.load_rom(BundledRom::Breakout).expect("load failed");
	
	for i in 0..100 {
		let filename = screenshots_dir.join(format!("{:04}.png", i));
		let filename_str = filename.to_string_lossy().to_string();
		let filename_cstr = CString::new(filename_str).expect("illegal filename");
		unsafe {
			ale.save_screen_png(&filename_cstr);
		}
		let legal_actions = ale.legal_action_set();
		let action = legal_actions[rand::thread_rng().gen_range(0, legal_actions.len())];
		ale.act(action);
	}
}
