use image;

use std::path::Path;
use std::fs::File;
use std::io::Read;

use ::types::TextureData;



pub struct Texture {
    //gfx_texture:u32
}

impl Texture {
    pub fn load<P: AsRef<Path>>(path: P) -> Cursor<Vec<u8>> {
        let mut buf = Vec::new();
        let fullpath = &Path::new("data").join(&path);
        let mut file = match File::open(&fullpath) {
            Ok(file) => file,
            Err(err) => {
                panic!("Can`t open file '{}' ({})", fullpath.display(), err);
            },
        };
        match file.read_to_end(&mut buf) {
            Ok(_) => Cursor::new(buf),
            Err(err) => {
                panic!("Can`t read file '{}' ({})", fullpath.display(), err);
            },
        }

        let image_buffer = image::load(cursor, image::PNG).unwrap().to_rgba();
        let (image_width, image_height) = image_buffer.dimensions();
        let texture_data=image_buffer.into_vec();

    }
}