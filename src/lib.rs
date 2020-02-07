
use std::ptr::null_mut;
use std::ffi::CStr;
use std::convert::TryInto;

pub struct Ale {
	ptr: *mut ale_sys::ALEInterface,
}
impl Ale {
	pub fn new() -> Ale {
		let ptr = unsafe { ale_sys::ALE_new() };
		assert!(ptr != null_mut());
		Ale {
			ptr
		}
	}

	// pub fn getString(ale: *mut ALEInterface, key: *const c_char) -> *const c_char; // TODO
	// pub fn getInt(ale: *mut ALEInterface, key: *const c_char) -> c_int; // TODO
	// pub fn getBool(ale: *mut ALEInterface, key: *const c_char) -> bool; // TODO
	// pub fn getFloat(ale: *mut ALEInterface, key: *const c_char) -> f32; // TODO
	// pub fn setString(ale: *mut ALEInterface, key: *const c_char, value: *const c_char) -> c_void; // TODO
	// pub fn setInt(ale: *mut ALEInterface, key: *const c_char, value: c_int) -> c_void; // TODO
	// pub fn setBool(ale: *mut ALEInterface, key: *const c_char, value: bool) -> c_void; // TODO
	// pub fn setFloat(ale: *mut ALEInterface, key: *const c_char, value: f32) -> c_void; // TODO

	/// Resets the Atari and loads a game.
	/// 
	/// After this call the game should be ready to play. This is necessary after changing a
	/// setting for the setting to take effect.
	pub fn load_rom(&mut self, rom_file: &CStr) {
		unsafe { ale_sys::loadROM(self.ptr, rom_file.as_ptr()); }
	}

	/// Applies an action to the game and returns the reward.
	/// 
	/// It is the user's responsibility to check if the game has ended and reset
	/// when necessary - this method will keep pressing buttons on the game over screen.
	pub fn act(&mut self, action: isize) -> isize {
		unsafe { ale_sys::act(self.ptr, action) }
	}

	/// Indicates if the game has ended.
	pub fn is_game_over(&mut self) -> bool {
		unsafe { ale_sys::game_over(self.ptr) }
	}

	/// Resets the game, but not the full system.
	pub fn reset_game(&mut self) {
		unsafe { ale_sys::reset_game(self.ptr); }
	}

	/// Returns the vector of modes available for the current game.
	///
	/// This should be called only after the rom is loaded.
	pub fn available_modes(&mut self) -> Vec<isize> {
		let size = unsafe { ale_sys::getAvailableModesSize(self.ptr) };
		assert!(size >= 0);
		let mut modes = vec![0; size as usize];
		unsafe { ale_sys::getAvailableModes(self.ptr, modes.as_mut_ptr()); }
		modes
	}
	pub fn set_mode(&mut self, mode: isize) {
		unsafe { ale_sys::setMode(self.ptr, mode); }
	}
	pub fn available_difficulties(&mut self) -> Vec<isize> {
		let size = unsafe { ale_sys::getAvailableDifficultiesSize(self.ptr) };
		assert!(size >= 0);
		let mut difficulties = vec![0; size as usize];
		unsafe { ale_sys::getAvailableDifficulties(self.ptr, difficulties.as_mut_ptr()); }
		difficulties
	}

	/// Sets the difficulty of the game.
	///
	/// This should be called only after the rom is loaded.
	/// 
	/// # Panics
	/// If the difficulty is not a valid difficulty TODO
	pub fn set_difficulty(&mut self, difficulty: isize) {
		unsafe { ale_sys::setDifficulty(self.ptr, difficulty); }
	}

	/// Returns the vector of legal actions. This should be called only after the ROM is loaded.
	pub fn legal_action_set(&mut self) -> Vec<isize> {
		let size = unsafe { ale_sys::getLegalActionSize(self.ptr) };
		assert!(size >= 0);
		let mut actions = vec![0; size as usize];
		unsafe { ale_sys::getLegalActionSet(self.ptr, actions.as_mut_ptr()); }
		actions
	}

	/// Returns the vector of the minimal set of actions needed to play the game.
	pub fn minimal_action_set(&mut self) -> Vec<isize> {
		let size = unsafe { ale_sys::getMinimalActionSize(self.ptr) };
		assert!(size >= 0);
		let mut actions = vec![0; size as usize];
		unsafe { ale_sys::getMinimalActionSet(self.ptr, actions.as_mut_ptr()); }
		actions
	}

	/// Returns the frame number since the loading of the ROM.
	pub fn frame_number(&mut self) -> isize {
		unsafe { ale_sys::getFrameNumber(self.ptr) }
	}

	/// Returns the remaining number of lives.
	pub fn lives(&mut self) -> isize {
		unsafe { ale_sys::lives(self.ptr) }
	}

	/// Returns the frame number since the start of the current episode.
	pub fn episode_frame_number(&mut self) -> isize {
		unsafe { ale_sys::getEpisodeFrameNumber(self.ptr) }
	}

	/// Writes the emulator's RAM contents to the buffer provided.
	/// 
	/// # Panics
	/// If the buffer is smaller than what [`ram_size()`](#func.ram_size) returns.
	pub fn get_ram(&mut self, ram: &mut [u8]) {
		assert!(ram.len() >= self.ram_size());
		unsafe { ale_sys::getRAM(self.ptr, ram.as_mut_ptr()); }
	}

	/// Get the size of the emulator's RAM, in bytes.
	pub fn ram_size(&mut self) -> usize {
		unsafe { ale_sys::getRAMSize(self.ptr) }.try_into().expect("invalid size")
	}

	/// Get the scren's width in pixels.
	pub fn screen_width(&mut self) -> usize {
		unsafe { ale_sys::getScreenWidth(self.ptr) }.try_into().expect("invalid size")
	}

	/// Get the scren's height in pixels.
	pub fn screen_height(&mut self) -> usize {
		unsafe { ale_sys::getScreenHeight(self.ptr) }.try_into().expect("invalid size")
	}

	/// Writes the screen's data to the buffer provided, in RGB format.
	///
	/// Pixel value at `x,y` is equal to `scren_data[y * screen_width() + x]`.
	///
	/// # Panics
	/// If the buffer is smaller than `screen_width() * screen_height() * 3`.
	pub fn get_screen_rgb(&mut self, screen_data: &mut [u8]) {
		assert!(screen_data.len() >= self.screen_width() * self.screen_height() * 3);
		unsafe { ale_sys::getScreenRGB(self.ptr, screen_data.as_mut_ptr()); }
	}

	/// Writes the screen's data to the buffer provided, in grayscale format, where `0 = black` and `255 = white`.
	///
	/// Pixel value at `x,y` is equal to `scren_data[y * screen_width() + x]`.
	///
	/// # Panics
	/// If the buffer is smaller than `screen_width() * screen_height()`.
	pub fn get_screen_grayscale(&mut self, screen_data: &mut [u8]) {
		assert!(screen_data.len() >= self.screen_width() * self.screen_height());
		unsafe { ale_sys::getScreenGrayscale(self.ptr, screen_data.as_mut_ptr()); }
	}

	/// Save the state of the system, to be restored using [`load_state()`](#func.load_state).
	pub fn save_state(&mut self) {
		unsafe { ale_sys::saveState(self.ptr); } 
	}

	/// Loads the state of the system that was saved by [`save_state()`](#func.save_state).
	pub fn load_state(&mut self) {
		unsafe { ale_sys::loadState(self.ptr); } 
	}

	// VVV TODO VVV
	// pub fn cloneState(&mut self) -> *mut ALEState;
	// pub fn restoreState(&mut self, state: *mut ALEState);
	// pub fn cloneSystemState(&mut self) -> *mut ALEState;
	// pub fn restoreSystemState(&mut self, state: *mut ALEState);
	// pub fn deleteState(state: *mut ALEState);
	// pub fn saveScreenPNG(&mut self, filename: *const c_char);

	// // Encodes the state as a raw bytestream. This may have multiple '\0' characters
	// // and thus should not be treated as a C string. Use encodeStateLen to find the length
	// // of the buffer to pass in, or it will be overrun as this simply memcpys bytes into the buffer.
	// pub fn encodeState(state: *mut ALEState, buf: *mut c_char, buf_len: c_int);
	// pub fn encodeStateLen(state: *mut ALEState) -> c_int;
	// pub fn decodeState(serialized: *const c_char, len: c_int) -> *mut ALEState;

	// // 0: Info, 1: Warning, 2: Error
	// pub fn setLoggerMode(mode: c_int);
}
impl Drop for Ale {
	fn drop(&mut self) {
		unsafe { ale_sys::ALE_del(self.ptr); }
	}
}
