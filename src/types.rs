use std;
use object_pool::growable::ID;

pub use std::time::SystemTime as Time;
pub use std::time::Duration;

pub type TextureData=Vec<u8>;

pub use image::{GrayImage, GrayAlphaImage, RgbImage, RgbaImage};
pub use storage::{RgbaTextureID};
pub use storage::{ObjectMeshID, ObjectLodID};

#[derive(Debug,Copy,Clone,Eq,PartialEq)]
pub enum ThreadSource{
    Render=0,
    Process=1,
    Algorithm=2
}

impl std::fmt::Display for ThreadSource{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self{
            ThreadSource::Render => write!(f, "Render"),
            ThreadSource::Process => write!(f, "Process"),
            ThreadSource::Algorithm => write!(f, "Algorithm"),
        }
    }
}

/*
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum MeshID {
    Terrain(ID)
}

*/