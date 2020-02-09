//! Rust encapsulation of the [Arcade Learning Environment](https://github.com/mgbellemare/Arcade-Learning-Environment).
//! 
//! This is currently a work in progress.
//! 
//! # Requirements
//! This library requires the same dependencies as the [cmake-rs](https://github.com/alexcrichton/cmake-rs) library. In other words, [CMake](https://cmake.org/) needs to be installed.
//! 
//! # Unsafety
//! Generally this libarary has tried to encapsulate and minimize unsafety, but there could still be some pain points that I've missed (especially regarding C++ exceptions). Be sure to report an issue if this is the case!

use std::ptr::null_mut;
use std::ffi::{CString, CStr};
use std::convert::TryInto;
use std::os::raw::c_int;
use std::io;

pub struct Ale {
	ptr: *mut ale_sys::ALEInterface,
	available_difficulties: Vec<i32>,
	available_modes: Vec<i32>,
	legal_actions: Vec<i32>,
	minimal_actions: Vec<i32>,
}
impl Ale {
	/// Creates a new interface to the Arcade Learning Environment., i.e. a new emulator insatnce.
	pub fn new() -> Ale {
		let ptr = unsafe { ale_sys::ALE_new() };
		assert!(ptr != null_mut());
		Ale {
			ptr,
			available_difficulties: vec![],
			available_modes: vec![],
			legal_actions: vec![],
			minimal_actions: vec![],
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

	/// Resets the Atari and loads a bundled game.
	///
	/// After this call the game should be ready to play. This is necessary after changing a
	/// setting for the setting to take effect.
	///
	/// Returns an error if there was an IO exception when saving the bundled ROM to a temporary directory.
	///
	/// # Examples
	/// ```
	/// # use ale::{Ale, BundledRom};
	/// let mut ale = Ale::new();
	/// ale.load_rom(BundledRom::Breakout);
	/// ale.act(1);
	/// assert_eq!(ale.is_game_over(), false);
	/// ```
	pub fn load_rom(&mut self, rom: BundledRom) -> io::Result<()> {
		// Save ROM to temp dir
		let dir = tempdir::TempDir::new("ale-rs")?;
		let rom_path = dir.path().join(rom.filename());
		std::fs::write(&rom_path, rom.data())?;

		// Call load_rom_file
		let rom_path_string = rom_path.to_string_lossy().to_string();
		let rom_path_c_str = CString::new(rom_path_string).expect("Invalid path");
		self.load_rom_file(&rom_path_c_str);
		Ok(())
	}

	/// Resets the Atari and loads a game from the file specified.
	/// 
	/// After this call the game should be ready to play. This is necessary after changing a
	/// setting for the setting to take effect.
	pub fn load_rom_file(&mut self, rom_file: &CStr) {
		unsafe { ale_sys::loadROM(self.ptr, rom_file.as_ptr()); }
	}

	/// Applies an action to the game and returns the reward.
	/// 
	/// It is the user's responsibility to check if the game has ended and reset
	/// when necessary - this method will keep pressing buttons on the game over screen.
	///
	/// # Panics
	/// If the action is not legal.
	pub fn act(&mut self, action: i32) -> i32 {
		assert!(self.legal_action_set().contains(&action), "Illegal action: {}", action);
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
	pub fn available_modes(&mut self) -> &[i32] {
		let size = unsafe { ale_sys::getAvailableModesSize(self.ptr) };
		assert!(size >= 0);
		self.available_modes.resize(size as usize, 0);
		unsafe { ale_sys::getAvailableModes(self.ptr, self.available_modes.as_mut_ptr()); }
		&self.available_modes
	}

	/// Sets the mode of the game.
	///
	/// This should be called only after the rom is loaded.
	///
	/// # Panics
	/// If the mode is invalid.
	pub fn set_mode(&mut self, mode: i32) {
		assert!(self.available_modes().contains(&mode), "Invalid mode: {}", mode);
		unsafe { ale_sys::setMode(self.ptr, mode); }
	}

	/// Returns the vector of difficulties available for the current game.
	///
	/// This should be called only after the rom is loaded.
	/// 
	/// Notice that there are 2 levers, the right and left switches. They are not tied to any specific player. In Venture, for example, we have the following interpretation for the difficulties:
	///
	/// | Skill Level | Switch Setting |
	/// |-------------|----------------|
	/// | 1           | left B/right B |
	/// | 2           | left B/right A |
	/// | 3           | left A/right B |
	/// | 4           | left A/right A |
	pub fn available_difficulties(&mut self) -> &[i32] {
		let size = unsafe { ale_sys::getAvailableDifficultiesSize(self.ptr) };
		assert!(size >= 0);
		self.available_difficulties.resize(size as usize, 0);
		unsafe { ale_sys::getAvailableDifficulties(self.ptr, self.available_difficulties.as_mut_ptr()); }
		&self.available_difficulties
	}

	/// Sets the difficulty of the game.
	///
	/// This should be called only after the rom is loaded.
	/// 
	/// # Panics
	/// If the difficulty is not a valid difficulty
	pub fn set_difficulty(&mut self, difficulty: i32) {
		assert!(self.available_difficulties().contains(&difficulty), "Invalid difficulty: {}", difficulty);
		unsafe { ale_sys::setDifficulty(self.ptr, difficulty); }
	}

	/// Returns the vector of legal actions. This should be called only after the ROM is loaded.
	pub fn legal_action_set(&mut self) -> &[i32] {
		let size = unsafe { ale_sys::getLegalActionSize(self.ptr) };
		assert!(size >= 0);
		self.legal_actions.resize(size as usize, 0);
		unsafe { ale_sys::getLegalActionSet(self.ptr, self.legal_actions.as_mut_ptr()); }
		&self.legal_actions
	}

	/// Returns the vector of the minimal set of actions needed to play the game.
	pub fn minimal_action_set(&mut self) -> &[i32] {
		let size = unsafe { ale_sys::getMinimalActionSize(self.ptr) };
		assert!(size >= 0);
		self.minimal_actions.resize(size as usize, 0);
		unsafe { ale_sys::getMinimalActionSet(self.ptr, self.minimal_actions.as_mut_ptr()); }
		&self.minimal_actions
	}

	/// Returns the frame number since the loading of the ROM.
	pub fn frame_number(&mut self) -> i32 {
		unsafe { ale_sys::getFrameNumber(self.ptr) as i32 }
	}

	/// Returns the remaining number of lives.
	pub fn lives(&mut self) -> i32 {
		unsafe { ale_sys::lives(self.ptr) }
	}

	/// Returns the frame number since the start of the current episode.
	pub fn episode_frame_number(&mut self) -> i32 {
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

	/// This makes a copy of the environment state. This copy does *not* include pseudorandomness, making it suitable for planning purposes. By contrast, see [`clone_system_state()`](#func.clone_system_state).
	pub fn clone_state(&mut self) -> AleState {
		AleState {
			ptr: unsafe { ale_sys::cloneState(self.ptr) },
		}
	}
	
	/// Reverse operation of [`clone_state()`](#func.clone_state). This does not restore pseudorandomness, so that repeated
	/// calls to [`restore_state()`](#func.restore_state) in the stochastic controls setting will not lead to the same outcomes.
	///
	/// By contrast, see [`restore_system_state()`](#func.restore_system_state).
	pub fn restore_state(&mut self, state: &AleState) {
		unsafe { ale_sys::restoreState(self.ptr, state.ptr); }
	}
	
	/// This makes a copy of the system & environment state, suitable for serialization. This includes pseudorandomness and so is *not* suitable for planning purposes.
	pub fn clone_system_state(&mut self) -> AleState {
		AleState {
			ptr: unsafe { ale_sys::cloneSystemState(self.ptr) },
		}
	}
	
	/// Reverse operation of [`clone_system_state()`](#func.clone_system_state).
	pub fn restore_system_state(&mut self, state: &AleState) {
		unsafe { ale_sys::restoreSystemState(self.ptr, state.ptr); }
	}

	/// Save the current screen as a png file
	/// 
	/// # Unsafety
	/// I am not sure, but this function may trigger undefined behaviour when a C++ exception is triggered.
	/// 
	/// To be safe, this function is marked as unsafe.
	pub unsafe fn save_screen_png(&mut self, filename: &CStr) {
		ale_sys::saveScreenPNG(self.ptr, filename.as_ptr());
	}

	/// Set logger mode
	pub fn set_logger_mode(mode: LoggerMode) {
		unsafe { ale_sys::setLoggerMode(mode as c_int); }
	}
}
impl Drop for Ale {
	fn drop(&mut self) {
		unsafe {
			let ptr = self.ptr;
			self.ptr = std::ptr::null_mut();
			ale_sys::ALE_del(ptr);
		}
	}
}

pub struct AleState {
	ptr: *mut ale_sys::ALEState,
}
impl AleState {
	/// Encodes the state as a raw bytestream.
	/// 
	/// # Panics
	/// If the length of `buf` is not large enough. Use [`encode_state_len()`](#func.encode_state_len) to get the needed length.
	pub fn encode_state(&self, buf: &mut [u8]) {
		assert!(buf.len() >= self.encode_state_len(), "Buffer not long enough to store encoded state. Expected {}, got {}", self.encode_state_len(), buf.len());
		unsafe { ale_sys::encodeState(self.ptr, buf.as_mut_ptr() as *mut _, buf.len() as c_int); }
	}

	/// Returns the length of the buffer needed to encode the state.
	///
	/// # Panics
	/// If the C API returns a negative size.
	pub fn encode_state_len(&self) -> usize {
		let size = unsafe { ale_sys::encodeStateLen(self.ptr) };
		assert!(size >= 0, "Invalid size: {}", size);
		size as usize
	}

	/// Decode state from a raw bytestream.
	///
	/// # Panics
	/// If the serialized length is too long to fit into a C integer.
	pub fn decode_state(serialized: &[u8]) -> AleState {
		let len: c_int = serialized.len().try_into().expect("Length too long");
		// TODO: Exceptions
		AleState {
			ptr: unsafe { ale_sys::decodeState(serialized.as_ptr() as *const _, len) },
		}
	}
}
impl Drop for AleState {
	fn drop(&mut self) {
		unsafe {
			let ptr = self.ptr;
			self.ptr = std::ptr::null_mut();
			ale_sys::deleteState(ptr);
		}
	}
}

pub enum LoggerMode {
	Info = 0,
	Warning = 1,
	Error = 2,
}

/// Enum of ROMs that come bundled with the libarary.
/// 
/// Note: Commented out ROMs are supported, but not bundled.
pub enum BundledRom {
	Adventure,
	AirRaid,
	Alien,
	Amidar,
	Assault,
	Asterix,
	Asteroids,
	Atlantis,
	BankHeist,
	BattleZone,
	BeamRider,
	Berzerk,
	Bowling,
	Boxing,
	Breakout,
	Carnival,
	Centipede,
	ChopperCommand,
	CrazyClimber,
	Defender,
	DemonAttack,
	// DonkeyKong,
	DoubleDunk,
	ElevatorAction,
	Enduro,
	FishingDerby,
	Freeway,
	// Frogger,
	Frostbite,
	// Galaxian,
	Gopher,
	Gravitar,
	Hero,
	IceHockey,
	JamesBond,
	JourneyEscape,
	Kaboom,
	Kangaroo,
	// Koolaid,
	// KeystoneKapers,
	// Kingkong,
	Krull,
	KungFuMaster,
	// LaserGates,
	// LostLuggage,
	MontezumaRevenge,
	// MrDo,
	MsPacman,
	NameThisGame,
	Phoenix,
	Pitfall,
	Pong,
	Pooyan,
	PrivateEye,
	QBert,
	RiverRaid,
	RoadRunner,
	RoboTank,
	Seaquest,
	// SirLancelot,
	Skiing,
	// Solaris,
	SpaceInvaders,
	StarGunner,
	Tennis,
	// Tetris,
	TimePilot,
	// Turmoil,
	// Trondead,
	Tutankham,
	UpNDown,
	Venture,
	VideoPinball,
	WizardOfWor,
	YarsRevenge,
	Zaxxon,
}
impl BundledRom {
	/// Returns the filename that the ROM should be named, in order for the ALE to pick up on it and
	/// use the correct settings.
	pub fn filename(&self) -> &'static str {
		use BundledRom::*;
		match self {
			Adventure => "adventure.bin",
			AirRaid => "air_raid.bin",
			Alien => "alien.bin",
			Amidar => "amidar.bin",
			Assault => "assault.bin",
			Asterix => "asterix.bin",
			Asteroids => "asteroids.bin",
			Atlantis => "atlantis.bin",
			BankHeist => "bank_heist.bin",
			BattleZone => "battle_zone.bin",
			BeamRider => "beam_rider.bin",
			Berzerk => "berzerk.bin",
			Bowling => "bowling.bin",
			Boxing => "boxing.bin",
			Breakout => "breakout.bin",
			Carnival => "carnival.bin",
			Centipede => "centipede.bin",
			ChopperCommand => "chopper_command.bin",
			CrazyClimber => "crazy_climber.bin",
			Defender => "defender.bin",
			DemonAttack => "demon_attack.bin",
			// DonkeyKong => ???,
			DoubleDunk => "double_dunk.bin",
			ElevatorAction => "elevator_action.bin",
			Enduro => "enduro.bin",
			FishingDerby => "fishing_derby.bin",
			Freeway => "freeway.bin",
			// Frogger => ???,
			Frostbite => "frostbite.bin",
			// Galaxian => ???,
			Gopher => "gopher.bin",
			Gravitar => "gravitar.bin",
			Hero => "hero.bin",
			IceHockey => "ice_hockey.bin",
			JamesBond => "jamesbond.bin",
			JourneyEscape => "journey_escape.bin",
			Kaboom => "kaboom.bin",
			Kangaroo => "kangaroo.bin",
			// Koolaid => ???,
			// KeystoneKapers => ???,
			// Kingkong => ???,
			Krull => "krull.bin",
			KungFuMaster => "kung_fu_master.bin",
			// LaserGates => ???,
			// LostLuggage => ???,
			MontezumaRevenge => "montezuma_revenge.bin",
			// MrDo => ???,
			MsPacman => "ms_pacman.bin",
			NameThisGame => "name_this_game.bin",
			Phoenix => "phoenix.bin",
			Pitfall => "pitfall.bin",
			Pong => "pong.bin",
			Pooyan => "pooyan.bin",
			PrivateEye => "private_eye.bin",
			QBert => "qbert.bin",
			RiverRaid => "riverraid.bin",
			RoadRunner => "road_runner.bin",
			RoboTank => "robotank.bin",
			Seaquest => "seaquest.bin",
			// SirLancelot => ???,
			Skiing => "skiing.bin",
			// Solaris => ???,
			SpaceInvaders => "space_invaders.bin",
			StarGunner => "star_gunner.bin",
			Tennis => "tennis.bin",
			// Tetris => ???,
			TimePilot => "time_pilot.bin",
			// Turmoil => ???,
			// Trondead => ???,
			Tutankham => "tutankham.bin",
			UpNDown => "up_n_down.bin",
			Venture => "venture.bin",
			VideoPinball => "video_pinball.bin",
			WizardOfWor => "wizard_of_wor.bin",
			YarsRevenge => "yars_revenge.bin",
			Zaxxon => "zaxxon.bin",
		}
	}

	/// Returns the raw binary data of the ROM.
	pub fn data(&self) -> &'static [u8] {
		use BundledRom::*;
		match self {
			Adventure => include_bytes!("../roms/adventure.bin"),
			AirRaid => include_bytes!("../roms/air_raid.bin"),
			Alien => include_bytes!("../roms/alien.bin"),
			Amidar => include_bytes!("../roms/amidar.bin"),
			Assault => include_bytes!("../roms/assault.bin"),
			Asterix => include_bytes!("../roms/asterix.bin"),
			Asteroids => include_bytes!("../roms/asteroids.bin"),
			Atlantis => include_bytes!("../roms/atlantis.bin"),
			BankHeist => include_bytes!("../roms/bank_heist.bin"),
			BattleZone => include_bytes!("../roms/battle_zone.bin"),
			BeamRider => include_bytes!("../roms/beam_rider.bin"),
			Berzerk => include_bytes!("../roms/berzerk.bin"),
			Bowling => include_bytes!("../roms/bowling.bin"),
			Boxing => include_bytes!("../roms/boxing.bin"),
			Breakout => include_bytes!("../roms/breakout.bin"),
			Carnival => include_bytes!("../roms/carnival.bin"),
			Centipede => include_bytes!("../roms/centipede.bin"),
			ChopperCommand => include_bytes!("../roms/chopper_command.bin"),
			CrazyClimber => include_bytes!("../roms/crazy_climber.bin"),
			Defender => include_bytes!("../roms/defender.bin"),
			DemonAttack => include_bytes!("../roms/demon_attack.bin"),
			// DonkeyKong => ???,
			DoubleDunk => include_bytes!("../roms/double_dunk.bin"),
			ElevatorAction => include_bytes!("../roms/elevator_action.bin"),
			Enduro => include_bytes!("../roms/enduro.bin"),
			FishingDerby => include_bytes!("../roms/fishing_derby.bin"),
			Freeway => include_bytes!("../roms/freeway.bin"),
			// Frogger => ???,
			Frostbite => include_bytes!("../roms/frostbite.bin"),
			// Galaxian => ???,
			Gopher => include_bytes!("../roms/gopher.bin"),
			Gravitar => include_bytes!("../roms/gravitar.bin"),
			Hero => include_bytes!("../roms/hero.bin"),
			IceHockey => include_bytes!("../roms/ice_hockey.bin"),
			JamesBond => include_bytes!("../roms/jamesbond.bin"),
			JourneyEscape => include_bytes!("../roms/journey_escape.bin"),
			Kaboom => include_bytes!("../roms/kaboom.bin"),
			Kangaroo => include_bytes!("../roms/kangaroo.bin"),
			// Koolaid => ???,
			// KeystoneKapers => ???,
			// Kingkong => ???,
			Krull => include_bytes!("../roms/krull.bin"),
			KungFuMaster => include_bytes!("../roms/kung_fu_master.bin"),
			// LaserGates => ???,
			// LostLuggage => ???,
			MontezumaRevenge => include_bytes!("../roms/montezuma_revenge.bin"),
			// MrDo => ???,
			MsPacman => include_bytes!("../roms/ms_pacman.bin"),
			NameThisGame => include_bytes!("../roms/name_this_game.bin"),
			Phoenix => include_bytes!("../roms/phoenix.bin"),
			Pitfall => include_bytes!("../roms/pitfall.bin"),
			Pong => include_bytes!("../roms/pong.bin"),
			Pooyan => include_bytes!("../roms/pooyan.bin"),
			PrivateEye => include_bytes!("../roms/private_eye.bin"),
			QBert => include_bytes!("../roms/qbert.bin"),
			RiverRaid => include_bytes!("../roms/riverraid.bin"),
			RoadRunner => include_bytes!("../roms/road_runner.bin"),
			RoboTank => include_bytes!("../roms/robotank.bin"),
			Seaquest => include_bytes!("../roms/seaquest.bin"),
			// SirLancelot => ???,
			Skiing => include_bytes!("../roms/skiing.bin"),
			// Solaris => ???,
			SpaceInvaders => include_bytes!("../roms/space_invaders.bin"),
			StarGunner => include_bytes!("../roms/star_gunner.bin"),
			Tennis => include_bytes!("../roms/tennis.bin"),
			// Tetris => ???,
			TimePilot => include_bytes!("../roms/time_pilot.bin"),
			// Turmoil => ???,
			// Trondead => ???,
			Tutankham => include_bytes!("../roms/tutankham.bin"),
			UpNDown => include_bytes!("../roms/up_n_down.bin"),
			Venture => include_bytes!("../roms/venture.bin"),
			VideoPinball => include_bytes!("../roms/video_pinball.bin"),
			WizardOfWor => include_bytes!("../roms/wizard_of_wor.bin"),
			YarsRevenge => include_bytes!("../roms/yars_revenge.bin"),
			Zaxxon => include_bytes!("../roms/zaxxon.bin"),
		}
	}
}
