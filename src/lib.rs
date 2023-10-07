#![allow(clippy::new_without_default)]

mod constants;
mod display;
mod entity;
mod macros;
mod state;
mod ui;

use constants::*;
pub use display::*;
pub use entity::*;
pub use macros::*;
pub use state::*;
pub use ui::*;
use wasm_bindgen::prelude::*;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub struct Game {
    /// The display, a terminal-like screen of text
    term: Terminal,
    /// The internal game state
    state: GameState,
}

/// The public interface exposed in Javascript by wasm-bindgen.
/// This is largely a re-export of the Terminal-like functionality
/// used to display the game state.
#[wasm_bindgen]
impl Game {
    /// Creates a new Game instance
    pub fn new() -> Self {
        //console_error_panic_hook::set_once();
        let state = GameState::new();

        let mut term = Terminal::new();
        term.message(bconcat!(
            b"##################################################
#                                                #
#                                                #
#                                                #
#       .dBBBBP dBBBBBBP dBBBBBb   dBBBBBb       #
#       BP                    BB       dBP       #
#       `BBBBb   dBP      dBP BB   dBBBBK        #
#          dBP  dBP      dBP  BB  dBP  BB        #
#     dBBBBP'  dBP      dBBBBBBB dBP  dB'        #
#                                                #
#               dBBBBBBP dBBBBBb    dBP dBBBBBb  #
#                            dBP            dB'  #
#                dBP     dBBBBK   dBP   dBBBP'   #
#               dBP     dBP  BB  dBP   dBP       #
#              dBP     dBP  dB' dBP   dBP        #
#                                                #
#   Version ",
            env!("CARGO_PKG_VERSION").as_bytes(),
            b"                                #
#                                                #
#                                                #
#                                                #
#               +----------------+               #
#               + Click to start |               #
#               +----------------+               #
#                                                #
##################################################"
        ));

        Self { term, state }
    }

    /// Returns the screen width (in tiles)
    pub fn width(&self) -> usize {
        WIDTH
    }

    /// Returns the screen height (in tiles)
    pub fn height(&self) -> usize {
        HEIGHT
    }

    /// Returns the number of tiles on a side in the tilesheet
    pub fn num_tiles(&self) -> usize {
        NUM_TILES
    }

    /// Returns the number of pixels on a side in a tile
    pub fn tile_size(&self) -> usize {
        TILE_SIZE
    }

    /// Returns the delay time used for drawing the screen
    pub fn delay_time(&self) -> usize {
        DELAY_TIME
    }

    /// Displays the introduction
    pub fn intro(&mut self) {
        self.term.message(bconcat!(
            b"Welcome, Captain, to your new command, the
HMS Venture. Your mission is to defend the galaxy
from the threat of the Klargons, Remulins,
Faringa and Berg. Defeat ",
            MISSION,
            b" enemies to win.

Your spaceship is well equipped with shields,
lasers and torpedoes. It can traverse great
distances at faster-than-light speeds!

Remember to keep track of your supplies,
especially the energy that powers your ship's
vital functions. Dock at a starbase or
investigate stars to resupply.

You're in command of an excellent crew, make sure
to take care of their morale by investigating
interesting planets on your journey.

Finally, watch out for astrophysical
phenomena such as supernovae and black holes!

Enter the HELP command for a listing of available
commands. Good luck!"
        ));

        self.term.update_console();
    }

    /// Displays the victory message
    pub fn win(&mut self) {
        let score = self.state.score();
        self.term.message(bconcat!(
            b"Well done, Captain, you've succeeded in
making the galaxy a safer place.

Your ship and crew have survived this difficult
mission. We thank you for your service.

You achieved a score of:


                         ",
            score,
            b"





To play again, refresh this webpage."
        ));
    }

    /// Displays the defeat message
    pub fn lose(&mut self) {
        let score = self.state.score();
        self.term.message(bconcat!(
            b"Unfortunately you have failed your mission
of making the galaxy a safer place.

Your ship has been destroyed, and only a handful
of crew members made it to the escape pods in
time. Your final log entry reads:

==================================================
",
            self.state.final_log(),
            b"
==================================================

You achieved a score of:


                         ",
            score,
            b"


To play again, refresh this webpage."
        ));
    }

    /// Returns a pointer to the display buffer
    pub fn screen(&self) -> *const u8 {
        self.term.screen()
    }

    /// Returns a pointer to the console part of the display buffer
    pub fn console(&self) -> *const u8 {
        self.term.console()
    }

    /// Overwrites the current cursor position with the given character
    pub fn input(&mut self, c: u16) {
        self.term.input(c);
    }

    /// Moves the cursor position one tile to the left
    pub fn left(&mut self) {
        self.term.left();
    }

    /// Moves the cursor position one tile to the right
    pub fn right(&mut self) {
        self.term.right();
    }

    /// Cycles forward through the command history
    pub fn down(&mut self) {
        self.term.down();
    }

    /// Cycles backward through the command history
    pub fn up(&mut self) {
        self.term.up();
    }

    /// Consumes a line of user commands and processes them
    pub fn enter(&mut self) -> u8 {
        let command = self.term.enter();
        self.state.process_command(&command, &mut self.term)
    }
}
