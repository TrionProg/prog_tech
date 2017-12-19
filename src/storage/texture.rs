use std;
use nes::{ErrorInfo,ErrorInfoTrait};
use image;

use object_pool::growable::ID;

use std::fs::File;
use std::io::{Read,Cursor};

use super::Error;
use super::Storage;
use super::TextureStorage;

pub trait TextureID {
    fn new(id:ID) -> Self;
    fn get_id(&self) -> ID;
}

/*
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct RgbTextureID(ID);

impl TextureID for RgbTextureID {
    fn new(id:ID) -> Self {RgbTextureID(id)}
    fn get_id(&self) -> ID {self.0}
}

*/

pub struct RgbaTexture {}

impl RgbaTexture {
    pub fn load(file_name:&str, storage:&Storage) -> Result<RgbaTextureID,Error> {
        let mut file = match File::open(file_name) {
            Ok(file) => file,
            Err(err) => return err!(Error::OpenImageFileError, file_name.to_string()),
        };

        let mut buf=Vec::with_capacity(1024*16);

        let cursor=match file.read_to_end(&mut buf) {
            Ok(_) => Cursor::new(buf),
            Err(err) => return err!(Error::ReadImageFileError, file_name.to_string()),
        };

        let image_buffer = image::load(cursor, image::PNG).unwrap().to_rgba();

        storage.load_texture(image_buffer)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct RgbaTextureID(ID);

impl TextureID for RgbaTextureID {
    fn new(id:ID) -> Self {RgbaTextureID(id)}
    fn get_id(&self) -> ID {self.0}
}



