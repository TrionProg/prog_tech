
use ::consts::MAP_SIZE;

pub struct Cursor {
    pub x:u32,
    pub z:u32,
    pub pos1:Option<(u32,u32)>,
    pub pos2:Option<(u32,u32)>
}

impl Cursor {
    pub fn new() -> Self {
        Cursor {
            x:0,
            z:0,
            pos1:None,
            pos2:None
        }
    }

    pub fn move_left(&mut self) {
        if self.x>0 {
            self.x-=1;
        }
    }

    pub fn move_right(&mut self) {
        if self.x<(MAP_SIZE-2) as u32 {
            self.x+=1;
        }
    }

    pub fn move_up(&mut self) {
        if self.z>0 {
            self.z-=1;
        }
    }

    pub fn move_down(&mut self) {
        if self.z<(MAP_SIZE-2) as u32 {
            self.z+=1;
        }
    }
}