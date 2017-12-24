use consts::MAP_SIZE;

#[derive(Clone,Copy)]
pub enum Tile {
    Air,
    Floor(usize),
    Wall(usize),
    Hole(usize)
}

pub struct Map {
    pub tiles:[[Tile;MAP_SIZE];MAP_SIZE]
}

impl Map {
    pub fn new() -> Self {
        Map {
            tiles:[[Tile::Air;MAP_SIZE];MAP_SIZE]
        }
    }
}

impl Tile {
    pub fn is_wall(&self) -> bool {
        match *self {
            Tile::Wall(_) => true,
            _ => false,
        }
    }

    pub fn is_hole(&self) -> bool {
        match *self {
            Tile::Hole(_) => true,
            _ => false,
        }
    }
}