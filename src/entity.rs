use crate::{constants::*, DisplayBytes};
use rand::{
    distributions::{Distribution, Standard},
    thread_rng, Rng,
};

/// Status of a spaceship
#[derive(Clone, Copy, PartialEq)]
pub struct Ship {
    pub energy: u8,
    pub shields: u8,
    pub torpedoes: u8,
    pub range: u8,
}

impl Ship {
    /// Randomly generate an enemy ship
    pub fn enemy() -> Self {
        let mut rng = thread_rng();

        let energy = rng.gen_range(20..DIFFICULTY);
        let shields = rng.gen_range(20..DIFFICULTY);
        let torpedoes = rng.gen_range(1..(DIFFICULTY / 20));
        let range = rng.gen_range(2..(DIFFICULTY / 20));

        Self {
            energy,
            shields,
            torpedoes,
            range,
        }
    }
}

/// Possible entities that might be encountered in
/// the depths of space
#[derive(Clone, Copy, PartialEq)]
pub enum Entity {
    BlackHole,
    Star,
    Planet,
    Base,
    Klargons(Ship),
    Remulins(Ship),
    Faringa(Ship),
    Berg(Ship),
}

impl Entity {
    /// Update the ship status of an enemy entity
    pub fn update(&self, ship: Ship) -> Option<Entity> {
        use Entity::*;
        match self {
            Klargons(_) => Some(Klargons(ship)),
            Remulins(_) => Some(Remulins(ship)),
            Faringa(_) => Some(Faringa(ship)),
            Berg(_) => Some(Berg(ship)),
            _ => None,
        }
    }
}

impl Distribution<Entity> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Entity {
        use Entity::*;
        match rng.gen_range(0..=7) {
            0 => BlackHole,
            1 => Star,
            2 => Planet,
            3 => Base,
            4 => Klargons(Ship::enemy()),
            5 => Remulins(Ship::enemy()),
            6 => Faringa(Ship::enemy()),
            _ => Berg(Ship::enemy()),
        }
    }
}

impl DisplayBytes for Entity {
    fn display_bytes(&self) -> Vec<u8> {
        use Entity::*;
        match self {
            BlackHole => b"Black hole".to_vec(),
            Star => b"Star".to_vec(),
            Planet => b"Planet".to_vec(),
            Base => b"Base".to_vec(),
            Klargons(_) => b"Klargons".to_vec(),
            Remulins(_) => b"Remulins".to_vec(),
            Faringa(_) => b"Faringa".to_vec(),
            Berg(_) => b"Berg".to_vec(),
        }
    }
}
