// Load bindings
import init, { Game } from './pkg/star_trip.js';
const wasm = await init();

// Global variables
let game,       // binding to Rust game engine
    canvas,     // HTML canvas used for rendering
    ctx,        // 2D context of canvas
    input,      // input element for keyboard
    tiles,      // image containing tileset
    width,      // screen width in tiles
    height,     // screen height in tiles
    tile_size,  // tile side in pixels
    num_tiles,  // number of tiles to a side in tilesheet
    delay_time, // dramatic pause for rendering in ms
    started,    // boolean for whether game has started
    delay,      // toggle for stylised drawing
    drawing;    // toggle during rendering

// Initialises game state and sets up browser event handlers
function gameSetup() {
  // Game configuration
  game = Game.new();
  width = game.width();
  height = game.height();
  tile_size = game.tile_size();
  num_tiles = game.num_tiles();
  delay_time = game.delay_time();

  // Canvas setup
  canvas = document.getElementById("game-canvas");
  canvas.width = tile_size * width;
  canvas.height = tile_size * height;
  ctx = canvas.getContext("2d");

  // Canvas styling
  resizeCanvas();
  window.addEventListener("resize", resizeCanvas);
  if (screen.orientation) {
    screen.orientation.addEventListener("change", resizeCanvas);
  } else {
    window.addEventListener("orientationchange", resizeCanvas);
  }

  // Input setup
  input = document.getElementById("game-input");

  // Load tileset
  tiles = new Image();
  tiles.src = "assets/tiles_16x16.png";

  // Initialise toggles
  started = false;
  delay = true;
  drawing = false;

  // Start things going once the tileset has loaded
  tiles.addEventListener("load", async function() {
    // Initial screen
    await drawScreen(false);

    // Install handler for clicks
    canvas.addEventListener("click", handleClicks);
  });
}

// Resizes canvas style
function resizeCanvas() {
  if (window.innerWidth >= canvas.width) {
    canvas.style.width = canvas.width + "px";
  } else if (window.innerWidth / window.innerHeight >= 2.0) {
    canvas.style.height = "90%";
  } else {
    canvas.style.width = "90%";
  }
}

// Draws image from tileset
function drawTile(tile, row, col) {
  const x = tile % num_tiles;
  const y = Math.floor(tile / num_tiles);
  ctx.drawImage(tiles, x * tile_size, y * tile_size, tile_size, tile_size,
                col * tile_size, row * tile_size, tile_size, tile_size);
}

// Brighten the tile
function brighten(row, col) {
  ctx.globalCompositeOperation = "lighten";
  ctx.fillStyle = "white";
  ctx.globalAlpha = 0.5;
  ctx.fillRect(col * tile_size, row * tile_size, tile_size, tile_size);
}

// Clears the screen
function clearScreen() {
  for (let i = 0; i < height; i++) {
    for (let j = 0; j < width; j++) {
      drawTile(0, i, j);
    }
  }
}

// Renders screen with a specified delay in between each
// tile to simulate a slow display refresh rate
async function drawScreen(sleep) {
  drawing = true;
  clearScreen();

  const ptr = game.screen();
  const screen = new Uint8Array(wasm.memory.buffer, ptr,
                                width * height);

  for (let i = 0; i < height; i++) {
    for (let j = 0; j < width; j++) {
      const val = screen[i * width + j];
      if (sleep && delay && !(val === 32 || val === 0)) {
        ctx.save();

        // Draw a brightened tile
        drawTile(val, i, j);
        brighten(i, j);

        // Pause for dramatic effect
        await new Promise((x) => setTimeout(x, delay_time));

        ctx.restore();
      }
      drawTile(val, i, j);
    }
  }

  // Reset after possible click modification
  delay = true;

  // Restore keyboard input
  drawing = false;
}

// Renders command line
function drawConsole() {
  const ptr = game.console();
  const console = new Uint8Array(wasm.memory.buffer, ptr, width);

  for (let j = 0; j < width; j++) {
    drawTile(console[j], height - 1, j);
  }
}

// Handles keyboard input
async function handleKeys(e) {
  if(drawing) {
    // Press a key to skip drawing line-printing animation
    delay = false;
  } else if([...e.key].length === 1) {
    game.input(e.key.charCodeAt(0)); // UTF-16 code unit
    drawConsole();
  } else if(e.key === "ArrowLeft") {
    game.left();
    drawConsole();
  } else if(e.key === "ArrowRight") {
    game.right();
    drawConsole();
  } else if(e.key === "ArrowUp") {
    game.up();
    drawConsole();
  } else if(e.key === "ArrowDown") {
    game.down();
    drawConsole();
  } else if(e.key === "Backspace") {
    game.left();
    game.input(32);
    game.left();
    drawConsole();
  } else if(e.key === "Delete") {
    game.input(32);
    drawConsole();
  } else if(e.key === "Enter") {
    status = game.enter();
    if(status > 0) {
      if(status == 1) {
        game.win();
      } else {
        game.lose();
      }
      // Game over
      window.removeEventListener("keydown", handleKeys);
    }
    await drawScreen(true);
  }
  input.value = "";
}

// Handles mouse clicks
async function handleClicks() {
  input.focus({preventScroll: true});

  if(!started) {
    // Start on first click
    started = true;
    game.intro();
    await drawScreen(true);

    // Start listening to keyboard input
    input.addEventListener("keydown", handleKeys);
  } else {
    // Click to skip drawing line-printing animation
    delay = false;
  }
}

gameSetup();
