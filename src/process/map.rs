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

    pub fn is_floor(&self, x:u32, z:u32) -> bool {
        self.tiles[x as usize][z as usize].is_floor()
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

    pub fn is_floor(&self) -> bool {
        match *self {
            Tile::Floor(_) => true,
            _ => false,
        }
    }
}