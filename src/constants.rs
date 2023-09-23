/// Number of systems on a side in the galaxy
pub const SYSTEMS: usize = 10;

/// Number of sectors on a side in a system
pub const SECTORS: usize = 10;

/// Game difficulty (higher is harder)
pub const DIFFICULTY: u8 = 100;

/// Victory condition
pub const MISSION: usize = 10;

/// Width of "terminal" display in tiles
pub const WIDTH: usize = 50;

/// Height of "terminal" display in tiles
pub const HEIGHT: usize = 25;

/// Pixels on a side in a given tile
pub const TILE_SIZE: usize = 16;

/// Tiles on a side in a tile-sheet
pub const NUM_TILES: usize = 16;

/// Time between drawing a tile (for dramatic effect)
pub const DELAY_TIME: usize = 20;

/// Extent of "terminal" excluding command line
pub const SCREEN: usize = WIDTH * (HEIGHT - 1);

/// Default command prompt
pub const COMMAND: &[u8] = b"COMMAND => ";

/// Width of command prompt
pub const CWIDTH: usize = COMMAND.len();

/// Number of lines of command history retained
pub const HIST: usize = 16;
