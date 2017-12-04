
use object_pool::growable::ID;

pub trait TextureID {
    fn new(id:ID) -> Self;
    fn get_id(&self) -> ID;
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct RgbTextureID(ID);

impl TextureID for RgbTextureID {
    fn new(id:ID) -> Self {RgbTextureID(id)}
    fn get_id(&self) -> ID {self.0}
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct RgbaTextureID(ID);

impl TextureID for RgbaTextureID {
    fn new(id:ID) -> Self {RgbaTextureID(id)}
    fn get_id(&self) -> ID {self.0}
}


