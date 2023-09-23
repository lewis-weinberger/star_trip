use crate::{bconcat, constants::*, index, nearby, DisplayBytes, Entity, Ship, Terminal};
use rand::{seq::SliceRandom, thread_rng, Rng};
use std::collections::HashSet;

/// The current state of the game including the player stats
/// and the state of all entities throughout the galaxy
pub struct GameState {
    galaxy: [Option<Entity>; SYSTEMS * SYSTEMS * SECTORS * SECTORS],
    logbook: Vec<Vec<u8>>,
    last_entry: Vec<u8>,
    page: usize,
    position: (usize, usize, usize, usize),
    player: Ship,
    mission: usize,
    date: usize,
}

impl GameState {
    /// Initialises a new game state
    pub fn new() -> Self {
        let mut galaxy = generate_galaxy();
        let logbook = vec![Vec::new()];
        let last_entry = b"COMPUTER ERROR: NO ENTRY AVAILABLE".to_vec();
        let page = 0;
        let mission = 0;
        let date = 0;

        let mut rng = thread_rng();
        let position = (
            rng.gen_range(0..SECTORS),
            rng.gen_range(0..SECTORS),
            rng.gen_range(0..SYSTEMS),
            rng.gen_range(0..SYSTEMS),
        );
        galaxy[index!(position)] = None;

        Self {
            galaxy,
            logbook,
            last_entry,
            page,
            position,
            player: Ship {
                energy: 255,
                shields: 255,
                torpedoes: 5,
                range: 7,
            },
            date,
            mission,
        }
    }

    /// Evolves the entities in the player's system by one time period
    fn evolve(&mut self, hostile: bool) {
        use Entity::*;

        let mut done = HashSet::new();
        let (x, y, xx, yy) = self.position;
        for i in 0..SECTORS {
            for j in 0..SECTORS {
                let sector = index!(j, i, xx, yy);
                if done.contains(&sector) {
                    continue;
                }
                let dr = (y.abs_diff(i) as f64).hypot(x.abs_diff(j) as f64);

                if let Some(
                    thing @ (Klargons(ship) | Remulins(ship) | Faringa(ship) | Berg(ship)),
                ) = self.galaxy[sector]
                {
                    // If this evolution is hostile, enemies within range
                    // can attack player
                    let ship = if hostile && (ship.range as f64) >= dr {
                        let mut rng = thread_rng();
                        let new;
                        let beam = if rng.gen_bool(0.5) {
                            let laser = rng.gen_range((ship.energy / 4)..(ship.energy / 2));
                            new = Ship {
                                energy: ship.energy - laser,
                                ..ship
                            };
                            laser
                        } else {
                            let number = rng.gen_range(0..ship.torpedoes);
                            new = Ship {
                                torpedoes: ship.torpedoes - number,
                                ..ship
                            };
                            50u8.saturating_mul(number)
                        };
                        let (damage, player) = self.fire(beam, self.player);
                        self.player = player;

                        self.record(bconcat!(
                            b"\nEnemy ",
                            thing,
                            b" have attacked!\nWe've taken ",
                            damage,
                            b" damage.\nRemaining ENERGY:  ",
                            self.player.energy,
                            b"\n          SHIELDS: ",
                            self.player.shields,
                            b"\n"
                        ));

                        new
                    } else {
                        ship
                    };

                    // Otherwise try to move in a random direction closer
                    // to the player (equivalent to speed 1)
                    for (n, m) in adjacent(j, i) {
                        let new = index!(n, m, xx, yy);
                        if self.galaxy[new].is_none() {
                            let ds = (y.abs_diff(m) as f64).hypot(x.abs_diff(n) as f64);
                            if ds < dr && ship.energy > 1 {
                                let ship = Ship {
                                    energy: ship.energy - 1,
                                    ..ship
                                };
                                self.galaxy[sector] = None;
                                self.galaxy[new] = thing.update(ship);

                                done.insert(new);

                                self.record(bconcat!(
                                    b"\nEnemy ",
                                    thing,
                                    b" moved to SECTOR: (",
                                    n,
                                    b",",
                                    m,
                                    b").\n"
                                ));
                                break;
                            }
                        }
                    }
                }
            }
        }

        self.date += 1;
    }

    /// Prints a helpful list of commands
    fn help(&self, term: &mut Terminal) {
        term.message(
            b"HELP - print this list of commands

MOVE s x y [X Y] - move towards sector
position (x, y) [optionally in system (X, Y)]
at speed s, where s is between 1 and 10

LASER e x y - fire lasers with energy e towards
position (x, y)

TORPEDO t x y - fire t torpedoes towards
position (x, y)

SHIELDS e - raise shields using energy e

SCAN - perform a short range scan of the system

SURVEY - perform a long range scan of the galaxy

INVESTIGATE - search for energy supplies

DOCK - dock your ship at a base to resupply

LOG n - print page n of the ship's log",
        );
        term.update_console();
    }

    /// Moves the player's ship as specified
    fn movement(&mut self, args: &[usize], term: &mut Terminal) {
        if let &[speed, x1, y1, ..] = args {
            if speed > 0 {
                let (x0, y0, xx0, yy0) = self.position;

                // First we process the intra-system movement
                let mut x = x0 as f64;
                let mut y = y0 as f64;
                loop {
                    let dx = x1 as f64 - x;
                    let dy = y1 as f64 - y;
                    let dr = dx.hypot(dy);

                    if self.player.energy == 0 || dr < 1e-2 {
                        break;
                    }

                    if speed as f64 > dr {
                        x = x1 as f64;
                        y = y1 as f64;
                        self.player.energy = self.player.energy.saturating_sub(dr.round() as u8);
                    } else {
                        x += (speed as f64 * dx / dr).round();
                        y += (speed as f64 * dy / dr).round();
                        self.player.energy = self.player.energy.saturating_sub(speed as u8);
                    }

                    self.record(bconcat!(
                        b"\nMoved to SECTOR: (",
                        x,
                        b", ",
                        y,
                        b") in SYSTEM: (",
                        xx0,
                        b", ",
                        yy0,
                        b").\nRemaining ENERGY: ",
                        self.player.energy,
                        b"\n"
                    ));

                    // Collisions damage shields or drain energy
                    if let Some(e) = self.galaxy[index!(x as usize, y as usize, xx0, yy0)] {
                        if e == Entity::BlackHole {
                            self.record(
                                b"\nYour ship fell into a black hole!
The hull lost integrity under the intense
gravitational pull and was crushed along
with any remaining crew onboard.\n",
                            );
                            self.player.energy = 0;
                            return;
                        }

                        let mut dmg = thread_rng().gen_range(0..DIFFICULTY);
                        if self.player.shields > dmg {
                            self.player.shields -= dmg;
                        } else {
                            dmg -= self.player.shields;
                            self.player.shields = 0;
                            self.player.energy = self.player.energy.saturating_sub(dmg);
                        }

                        self.record(bconcat!(
                            b"\nCollided with: ",
                            e,
                            b"!\nRemaining ENERGY: ",
                            self.player.energy,
                            b"\nRemaining SHIELDS: ",
                            self.player.shields,
                            b"\n"
                        ));
                    }

                    self.evolve(true);
                }

                let mut xx = xx0 as f64;
                let mut yy = yy0 as f64;
                if let &[_, _, _, xx1, yy1] = args {
                    // Finally we process the inter-system movement.
                    // We do not check for collisions here, as inter-system
                    // space is assumed to be sparse. We also do not evolve
                    // the entities any further (to save a few cycles).
                    loop {
                        let dx = xx1 as f64 - xx;
                        let dy = yy1 as f64 - yy;
                        let dr = dx.hypot(dy);

                        if self.player.energy == 0 || dr < 1e-2 {
                            break;
                        }

                        // It costs 10 times the energy to move between systems
                        // compared to moving between sectors
                        if speed as f64 > dr {
                            xx = xx1 as f64;
                            yy = yy1 as f64;
                            self.player.energy =
                                self.player.energy.saturating_sub((10.0 * dr.round()) as u8);
                        } else {
                            xx += (speed as f64 * dx / dr).round();
                            yy += (speed as f64 * dy / dr).round();

                            self.player.energy =
                                self.player.energy.saturating_sub(10 * speed as u8);
                        }

                        self.record(bconcat!(
                            b"\nMoved to SECTOR: (",
                            x,
                            b", ",
                            y,
                            b") in SYSTEM: (",
                            xx,
                            b", ",
                            yy,
                            b").\nRemaining ENERGY: ",
                            self.player.energy,
                            b"\n"
                        ));
                    }
                }

                self.position = (x as usize, y as usize, xx as usize, yy as usize);
                self.scan(term);
                return;
            }
        }

        // Incorrect arguments
        term.message(
            b"MOVE requires at least three positive
numeric arguments:

    MOVE s x y [X Y]

The speed s must be greater than zero.

Run HELP for more commands.",
        );
        term.update_console();
    }

    /// Calculates a hit on another ship
    fn fire(&mut self, beam: u8, ship: Ship) -> (u8, Ship) {
        let total = ship.energy + ship.shields;
        let damage = if total > beam { beam } else { total };
        let rem = if ship.shields > damage {
            0
        } else {
            damage - ship.shields
        };

        let shields = if rem > 0 { 0 } else { ship.shields - damage };
        let energy = ship.energy.saturating_sub(rem);
        let ship = Ship {
            energy,
            shields,
            ..ship
        };
        (damage, ship)
    }

    /// Calculates a hit on an enemy ship and updates galaxy
    fn hit(&mut self, sector: usize, beam: u8, enemy: Entity, ship: Ship) -> (u8, bool) {
        let (damage, ship) = self.fire(beam, ship);

        let destroyed;
        self.galaxy[sector] = if ship.energy > 0 {
            destroyed = false;
            enemy.update(ship)
        } else {
            self.record(bconcat!(b"\nEnemy ", enemy, b" destroyed!\n"));
            self.mission += 1;
            destroyed = true;
            None
        };

        (damage, destroyed)
    }

    /// Fires the player's lasers
    fn laser(&mut self, sector: usize, beam: u8, enemy: Entity, ship: Ship) -> Vec<u8> {
        let (damage, destroyed) = self.hit(sector, beam, enemy, ship);

        // Player loses all energy from beam (even if damage < beam)
        self.player.energy = self.player.energy.saturating_sub(beam);

        let msg = if destroyed {
            b"Their ship has been destroyed.\n"
        } else {
            b"                               "
        };

        bconcat!(
            b"\nHit ",
            enemy,
            b" with lasers inflicting ",
            damage,
            b" damage!\n",
            msg
        )
        .to_vec()
    }

    /// Fires the player's torpedoes as specified
    fn torpedo(&mut self, sector: usize, number: u8, enemy: Entity, ship: Ship) -> Vec<u8> {
        let number = if number > self.player.torpedoes {
            self.player.torpedoes
        } else {
            number
        };

        // Torpedoes do 100 damage down-weighted by the difficulty setting
        let beam = (number as f64) * 100.0 * (255.0 - DIFFICULTY as f64) / 255.0;
        let (damage, destroyed) = self.hit(sector, beam as u8, enemy, ship);

        self.player.torpedoes = self.player.torpedoes.saturating_sub(number);

        let msg = if destroyed {
            b"Their ship has been destroyed.\n"
        } else {
            b"                               "
        };

        bconcat!(
            b"\nHit ",
            enemy,
            b" with ",
            number,
            b" torpedo(es),\ninflicting ",
            damage,
            b" damage!\n",
            msg
        )
        .to_vec()
    }

    /// Fires the player's weapons as specified
    fn weapon(
        &mut self,
        args: &[usize],
        term: &mut Terminal,
        weapon: fn(&mut GameState, usize, u8, Entity, Ship) -> Vec<u8>,
        name: &[u8],
    ) {
        use Entity::*;
        if let &[amount, x, y] = args {
            if amount > 0 {
                let (j, i, xx, yy) = self.position;
                let sector = index!(x, y, xx, yy);
                let r = (x.abs_diff(j) as f64).hypot(y.abs_diff(i) as f64) as u8;
                let msg = if let Some(
                    thing @ (Klargons(ship) | Remulins(ship) | Faringa(ship) | Berg(ship)),
                ) = self.galaxy[sector]
                {
                    if self.player.range >= r {
                        weapon(self, sector, amount as u8, thing, ship)
                    } else {
                        bconcat!(b"(", x, b", ", y, b") out of range!").to_vec()
                    }
                } else {
                    bconcat!(b"Nothing to target at (", x, b", ", y, b")!").to_vec()
                };

                self.record(&msg);
                term.message(&msg);
                term.update_console();
                self.evolve(true);
                return;
            }
        }

        // Incorrect arguments
        term.message(bconcat!(
            name,
            b" requires three positive
numeric arguments:

",
            name,
            b" n x y

where n must be greater than zero.

Run HELP for more commands."
        ));
        term.update_console();
    }

    /// Raises the player's shields as specified
    fn shields(&mut self, args: &[usize], term: &mut Terminal) {
        if let &[energy] = args {
            let energy = energy as u8;
            if self.player.shields < 255 && energy <= self.player.energy {
                self.player.shields = self.player.shields.saturating_add(energy);
                self.player.energy = self.player.energy.saturating_sub(energy);

                let msg = bconcat!(
                    b"\nEnergy diverted to shields:\nENERGY:  ",
                    self.player.energy,
                    b"\nSHIELDS: ",
                    self.player.shields,
                    b"\n"
                );
                term.message(msg);
                self.record(msg);
                term.update_console();

                self.evolve(true);
                return;
            }
        }

        // Incorrect arguments
        term.message(
            b"SHIELDS command requires one positive argument!

Cannot raise shields beyond 255 energy.",
        );
        term.update_console();
    }

    /// Prints a star chart for the current system
    fn scan(&mut self, term: &mut Terminal) {
        use Entity::*;
        let legend: [(&[u8], u8); 8] = [
            (b"BLACK HOLE", 0x07),
            (b"STAR", 0x08),
            (b"PLANET", 0x09),
            (b"BASE", 0x0B),
            (b"KLARGONS", 0x03),
            (b"REMULINS", 0x04),
            (b"FARINGA", 0x05),
            (b"BERG", 0x06),
        ];
        let mut out = vec![];
        let mut enemies: usize = 0;
        let (x, y, xx, yy) = self.position;
        out.extend_from_slice(b"\n\n    0 1 2 3 4 5 6 7 8 9");
        out.extend_from_slice(b"        PLAYER:     ");
        out.push(if self.player.energy > 127 { 0x01 } else { 0x02 });
        for i in 0..SECTORS {
            out.extend_from_slice(bconcat!(b"\n  ", i));
            for j in 0..SECTORS {
                out.push(b' ');
                if j == x && i == y {
                    out.push(if self.player.energy > 127 { 0x01 } else { 0x02 });
                } else {
                    out.push(match self.galaxy[index!(j, i, xx, yy)] {
                        None => 0xFA,
                        Some(BlackHole) => 0x07,
                        Some(Star) => 0x08,
                        Some(Planet) => 0x09,
                        Some(Base) => 0x0B,
                        Some(Klargons(_)) => {
                            enemies += 1;
                            0x03
                        }
                        Some(Remulins(_)) => {
                            enemies += 1;
                            0x04
                        }
                        Some(Faringa(_)) => {
                            enemies += 1;
                            0x05
                        }
                        Some(Berg(_)) => {
                            enemies += 1;
                            0x06
                        }
                    });
                }
            }

            if i < legend.len() {
                out.extend_from_slice(b"        ");
                out.extend_from_slice(legend[i].0);
                out.extend_from_slice(b":");
                out.resize(out.len() + 11 - legend[i].0.len(), b' ');
                out.push(legend[i].1);
            }
        }

        out.extend_from_slice(bconcat!(
            b"\n\n\n SECTOR:    (",
            x,
            b", ",
            y,
            b")\n SYSTEM:    (",
            xx,
            b", ",
            yy,
            b")\n ENERGY:    ",
            self.player.energy,
            b"\n SHIELDS:   ",
            self.player.shields,
            b"\n TORPEDOES: ",
            self.player.torpedoes,
            b"\n DATE:      ",
            self.date,
            b"\n ENEMIES:   ",
            enemies,
            b"\n MISSION:   ",
            self.mission
        ));

        self.record(bconcat!(
            b"\nScan completed: ",
            enemies,
            b" enemies detected in system!\n"
        ));
        term.message(&out);
        term.update_console();
    }

    /// Prints a system chart for the galaxy
    fn survey(&self, term: &mut Terminal) {
        use Entity::*;
        let mut out = vec![];
        let (_, _, xx, yy) = self.position;
        out.extend_from_slice(b"\n\n     0   1   2   3   4   5   6   7   8   9");
        for i in 0..SYSTEMS {
            out.extend_from_slice(bconcat!(b"\n  ", i));
            for j in 0..SYSTEMS {
                out.push(b' ');

                // Scan adjacent systems
                if (j + 1 >= xx && j <= xx + 1) && (i + 1 >= yy && i <= yy + 1) {
                    let start = index!(0, 0, j, i);
                    let system = &self.galaxy[start..(start + SECTORS * SECTORS)];

                    let enemies: u8 = system
                        .iter()
                        .flatten()
                        .map(|n| match n {
                            Klargons(_) | Remulins(_) | Faringa(_) | Berg(_) => 1,
                            _ => 0,
                        })
                        .sum();
                    let bases: u8 = system
                        .iter()
                        .flatten()
                        .map(|n| match n {
                            Base => 1,
                            _ => 0,
                        })
                        .sum();
                    let stars: u8 = system
                        .iter()
                        .flatten()
                        .map(|n| match n {
                            Star => 1,
                            _ => 0,
                        })
                        .sum();

                    // There shouldn't be more than 9 of any of these
                    out.extend_from_slice(bconcat!(enemies, bases, stars));
                } else {
                    out.extend_from_slice(b"***");
                }
            }
        }
        out.extend_from_slice(
            b"\n\n\n    XYZ (SYSTEM TOTALS)\n    |||\n    \
                                ||+-> STARS\n    |+--> BASES\n    +---> ENEMIES\n",
        );

        term.message(&out);
        term.update_console();
    }

    /// Investigates adjacent stars or planets
    fn investigate(&mut self, term: &mut Terminal) {
        if let Some((x, y, thing)) = nearby!(self, Planet, Star, BlackHole) {
            let energy = thread_rng().gen_range(0..DIFFICULTY);
            term.message(bconcat!(
                b"Investigated nearby ",
                thing,
                b".\nDiscovered ",
                energy,
                b" energy crystals!"
            ));
            self.record(bconcat!(
                b"\nInvestigated nearby ",
                thing,
                b".\nAt SECTOR (",
                x,
                b", ",
                y,
                b").\nDiscovered ",
                energy,
                b" energy crystals!\n"
            ));
            self.player.energy = self.player.energy.saturating_add(energy);
        } else {
            term.message(b"Nothing interesting nearby, unable to investigate!");
        }
        self.evolve(true);
        term.update_console();
    }

    /// Docks the player's ship at an adjacent starbase
    fn dock(&mut self, term: &mut Terminal) {
        if let Some((x, y, _)) = nearby!(self, Base) {
            term.message(
                b"Docked with nearby base.
Energy and shields restored!
Protected from hostiles until next move.",
            );
            self.record(bconcat!(
                b"\nDocked with base in SECTOR: (",
                x,
                b", ",
                y,
                b").\nEnergy and shields restored!\n"
            ));
            self.player.shields = 255;
            self.player.energy = 255;
            self.evolve(false);
        } else {
            term.message(b"No bases nearby, unable to dock!");
            self.evolve(true);
        }
        term.update_console();
    }

    /// Displays a page of the log
    fn log(&self, args: &[usize], term: &mut Terminal) {
        match args {
            [] => term.message(bconcat!(
                b"Captain's Log [",
                self.page + 1,
                b" / ",
                self.logbook.len(),
                b"]\n\n",
                self.logbook[self.page].as_slice()
            )),
            [i] if *i < self.logbook.len() + 1 && *i > 0 => term.message(bconcat!(
                b"Captain's Log [",
                i,
                b" / ",
                self.logbook.len(),
                b"]\n\n",
                self.logbook[*i - 1].as_slice()
            )),
            _ => term.message(b"Log page not found!"),
        };
        term.update_console();
    }

    /// Writes an entry to the log
    fn record(&mut self, entry: &[u8]) {
        self.last_entry = entry.to_vec();
        let mut latest = &mut self.logbook[self.page];
        for &c in entry.iter() {
            // Make sure log page fits on the screen
            let length = latest.iter().filter(|&c| *c == b'\n').count();
            if length >= HEIGHT - 4 {
                self.logbook.push(Vec::new());
                self.page += 1;
                latest = &mut self.logbook[self.page];
            }
            latest.push(c);
        }
    }

    /// Returns the final entry in the log
    pub fn final_log(&self) -> &[u8] {
        &self.last_entry
    }

    /// Determines the player score
    pub fn score(&self) -> usize {
        let date = (100.0 * (-(self.date as f64) / 100.0).exp()) as usize;
        let mission = 5 * self.mission.pow(2);
        let energy = 4 * (self.player.energy as usize);
        let shields = 3 * (self.player.shields as usize);
        mission + date + energy + shields
    }

    /// Parses user input and dispatches to relevant methods
    pub fn process_command(&mut self, command: &[u8], term: &mut Terminal) -> u8 {
        // Split words on whitespace
        let words: Vec<&[u8]> = command
            .split(|c| c.is_ascii_whitespace())
            .filter(|s| !s.is_empty())
            .collect();
        if words.is_empty() {
            return 0;
        }

        // Process any numeric arguments.
        // Note: this will quietly ignore any non-numeric arguments
        // that are interspersed with the numeric arguments
        let args: Vec<usize> = words
            .iter()
            .flat_map(|n| String::from_utf8_lossy(n).parse::<usize>())
            .collect();

        // Dispatch based on command
        match String::from_utf8_lossy(words[0])
            .to_ascii_lowercase()
            .as_str()
        {
            "help" | "h" => self.help(term),
            "move" | "m" => self.movement(&args, term),
            "laser" | "l" => self.weapon(&args, term, GameState::laser, b"LASER"),
            "torpedo" | "t" => self.weapon(&args, term, GameState::torpedo, b"TORPEDO"),
            "shields" | "sh" => self.shields(&args, term),
            "scan" | "sc" => self.scan(term),
            "survey" | "su" => self.survey(term),
            "investigate" | "i" => self.investigate(term),
            "dock" | "d" => self.dock(term),
            "log" => self.log(&args, term),
            "quit" | "q" => {
                self.player.energy = 0;
            }
            _ => {
                term.message(bconcat!(
                    b"Unrecognised command:\n\n    '",
                    words[0],
                    b"'\n\nTry the HELP command for a list of possible\ncommands!"
                ));
                term.update_console();
            }
        };

        // Player wins if mission is completed,
        // loses if all energy is drained,
        // otherwise game continues
        if self.mission == MISSION {
            1
        } else if self.player.energy == 0 {
            2
        } else {
            0
        }
    }
}

/// Randomly generate a galaxy full of enemies and other entities
fn generate_galaxy() -> [Option<Entity>; SYSTEMS * SYSTEMS * SECTORS * SECTORS] {
    use Entity::*;

    let mut galaxy = [None; SYSTEMS * SYSTEMS * SECTORS * SECTORS];
    let difficulty = ((DIFFICULTY as f64 / 255.0) - 1.0).exp();

    let mut rng = thread_rng();
    for i in 0..SYSTEMS {
        for j in 0..SYSTEMS {
            let mut enemies = 0;
            let mut others = 0;
            let emax = (9.0 * rng.gen::<f64>() * difficulty) as usize;
            let omax = (9.0 * rng.gen::<f64>() * difficulty) as usize;

            for y in 0..SECTORS {
                for x in 0..SECTORS {
                    if rng.gen_bool(1.0 / 5.0) {
                        galaxy[index!(x, y, j, i)] = match rng.gen() {
                            e @ (BlackHole | Star | Planet | Base) if others < omax => {
                                others += 1;
                                Some(e)
                            }
                            e @ (Klargons(_) | Remulins(_) | Faringa(_) | Berg(_))
                                if enemies < emax =>
                            {
                                enemies += 1;
                                Some(e)
                            }
                            _ => None,
                        };
                    }
                }
            }

            // We've filled in row-major order, which will bias where we've
            // placed our enemies. To fix that we shuffle the entire system
            let start = index!(0, 0, j, i);
            galaxy[start..(start + SECTORS * SECTORS)].shuffle(&mut rng);
        }
    }
    galaxy
}

/// Iterator over adjacent sector positions
struct Adjacent {
    coords: Vec<(usize, usize)>,
}

/// Create a new Adjacent iterator for the given coordinate
fn adjacent(x: usize, y: usize) -> Adjacent {
    let xmin = x.saturating_sub(1);
    let ymin = y.saturating_sub(1);
    let xmax = if x + 3 >= SECTORS { SECTORS - 1 } else { x + 3 };
    let ymax = if y + 3 >= SECTORS { SECTORS - 1 } else { y + 3 };

    let mut coords = Vec::new();
    for i in xmin..xmax {
        for j in ymin..ymax {
            if (i, j) != (x, y) {
                coords.push((i, j));
            }
        }
    }

    Adjacent { coords }
}

impl Iterator for Adjacent {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        self.coords.pop()
    }
}
