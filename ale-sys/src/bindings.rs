// TODO

//use libc::c_int;
// TODO Fix
#[allow(non_camel_case_types)]
pub type c_int = isize;
// TODO Fix
#[allow(non_camel_case_types)]
pub type c_char = i8;
use std::ffi::c_void;

#[repr(C)]
pub struct ALEInterface {
	inner: u32
}

#[repr(C)]
pub struct ALEState {
	inner: u32
}

extern "C" {
	pub fn ALE_new() -> *mut ALEInterface;
	pub fn ALE_del(ale: *mut ALEInterface);
	pub fn getString(ale: *mut ALEInterface, key: *const c_char) -> *const c_char;
	pub fn getInt(ale: *mut ALEInterface, key: *const c_char) -> c_int;
	pub fn getBool(ale: *mut ALEInterface, key: *const c_char) -> bool;
	pub fn getFloat(ale: *mut ALEInterface, key: *const c_char) -> f32;
	pub fn setString(ale: *mut ALEInterface, key: *const c_char, value: *const c_char);
	pub fn setInt(ale: *mut ALEInterface, key: *const c_char, value: c_int);
	pub fn setBool(ale: *mut ALEInterface, key: *const c_char, value: bool);
	pub fn setFloat(ale: *mut ALEInterface, key: *const c_char, value: f32);
	pub fn loadROM(ale: *mut ALEInterface, rom_file: *const c_char);
	pub fn act(ale: *mut ALEInterface, action: c_int) -> c_int;
	pub fn game_over(ale: *const ALEInterface) -> bool;
	pub fn reset_game(ale: *mut ALEInterface);
	pub fn getAvailableModes(ale: *mut ALEInterface, availableModes: *const c_int);
	pub fn getAvailableModesSize(ale: *mut ALEInterface) -> c_int;
	pub fn setMode(ale: *mut ALEInterface, mode: c_int);
	pub fn getAvailableDifficulties(ale: *mut ALEInterface, availableDifficulties: *const c_int);
	pub fn getAvailableDifficultiesSize(ale: *mut ALEInterface) -> c_int;
	pub fn setDifficulty(ale: *mut ALEInterface, difficulty: c_int);
	pub fn getLegalActionSet(ale: *mut ALEInterface, actions: *const c_int);
	pub fn getLegalActionSize(ale: *mut ALEInterface) -> c_int;
	pub fn getMinimalActionSet(ale: *mut ALEInterface, actions: *const c_int);
	pub fn getMinimalActionSize(ale: *mut ALEInterface) -> c_int;
	pub fn getFrameNumber(ale: *mut ALEInterface) -> c_int;
	pub fn lives(ale: *mut ALEInterface) -> c_int;
	pub fn getEpisodeFrameNumber(ale: *mut ALEInterface) -> c_int;
	pub fn getScreen(ale: *mut ALEInterface, screen_data: *mut u8);
	pub fn getRAM(ale: *mut ALEInterface, ram: *mut u8);
	pub fn getRAMSize(ale: *mut ALEInterface) -> c_int;
	pub fn getScreenWidth(ale: *mut ALEInterface) -> c_int;
	pub fn getScreenHeight(ale: *mut ALEInterface) -> c_int;

	pub fn getScreenRGB(ale: *mut ALEInterface, output_buffer: *mut u8);

	pub fn getScreenGrayscale(ale: *mut ALEInterface, output_buffer: *mut u8);

	pub fn saveState(ale: *mut ALEInterface);
	pub fn loadState(ale: *mut ALEInterface);
	pub fn cloneState(ale: *mut ALEInterface) -> *mut ALEState;
	pub fn restoreState(ale: *mut ALEInterface, state: *mut ALEState);
	pub fn cloneSystemState(ale: *mut ALEInterface) -> *mut ALEState;
	pub fn restoreSystemState(ale: *mut ALEInterface, state: *mut ALEState);
	pub fn deleteState(state: *mut ALEState);
	pub fn saveScreenPNG(ale: *mut ALEInterface, filename: *const c_char);

	// Encodes the state as a raw bytestream. This may have multiple '\0' characters
	// and thus should not be treated as a C string. Use encodeStateLen to find the length
	// of the buffer to pass in, or it will be overrun as this simply memcpys bytes into the buffer.
	pub fn encodeState(state: *mut ALEState, buf: *mut c_char, buf_len: c_int);
	pub fn encodeStateLen(state: *mut ALEState) -> c_int;
	pub fn decodeState(serialized: *const c_char, len: c_int) -> *mut ALEState;

	// 0: Info, 1: Warning, 2: Error
	pub fn setLoggerMode(mode: c_int);
}
