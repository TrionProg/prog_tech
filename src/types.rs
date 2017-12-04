
use object_pool::growable::ID;


pub type TextureData=Vec<u8>;

pub use image::{GrayImage, GrayAlphaImage, RgbImage, RgbaImage};
pub use storage::{RgbTextureID, RgbaTextureID};

/*
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum MeshID {
    Terrain(ID)
}

*/