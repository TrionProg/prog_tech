use consts::MAP_SIZE;

#[derive(Clone,Copy)]
pub enum Tile {
    Air,
    Floor(usize),
    Wall(usize),
    Hole(usize)
}

pub struct Map {
    pub tiles:[[Tile;MAP_SIZE];MAP_SIZE],
    pub marks:[[u32;MAP_SIZE];MAP_SIZE],
    last_mark:u32,
}

impl Map {
    pub fn new() -> Self {
        Map {
            tiles:[[Tile::Air;MAP_SIZE];MAP_SIZE],
            marks:[[0;MAP_SIZE];MAP_SIZE],
            last_mark:0,
        }
    }

    pub fn is_floor(&self, x:u32, z:u32) -> bool {
        self.tiles[x as usize][z as usize].is_floor()
    }

    pub fn is_obstracle(&self, x:u32, z:u32) -> bool {
        !self.is_floor(x,z)
    }

    pub fn is_marked(&self, x:u32, z:u32, mark:u32) -> bool {
        self.marks[x as usize][z as usize]==mark
    }

    pub fn mark(&mut self, x:u32, z:u32, mark:u32) {
        self.marks[x as usize][z as usize]=mark;
    }

    pub fn get_mark(&mut self) -> u32 {
        self.last_mark+=1;
        self.last_mark
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