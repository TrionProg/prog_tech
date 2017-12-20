
#[derive(Clone,Copy)]
pub enum Tile {
    Air,
    Floor(usize),
    Wall(usize),
    Hole(usize)
}

pub struct Map {
    pub tiles:[[Tile;18];18]
}

impl Map {
    pub fn new() -> Self {
        Map {
            tiles:[[Tile::Air;18];18]
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