mod bindings;

pub use bindings::root::{
	act,
	ale::{ALEInterface, ALEState},
	cloneState, cloneSystemState, decodeState, deleteState, encodeState, encodeStateLen, game_over,
	getAvailableDifficulties, getAvailableDifficultiesSize, getAvailableModes, getAvailableModesSize, getBool,
	getEpisodeFrameNumber, getFloat, getFrameNumber, getInt, getLegalActionSet, getLegalActionSize,
	getMinimalActionSet, getMinimalActionSize, getRAM, getRAMSize, getScreen, getScreenGrayscale, getScreenHeight,
	getScreenRGB, getScreenWidth, getString, lives, loadROM, loadState, reset_game, restoreState, restoreSystemState,
	saveScreenPNG, saveState, setBool, setDifficulty, setFloat, setInt, setLoggerMode, setMode, setString, ALE_del,
	ALE_new,
};
