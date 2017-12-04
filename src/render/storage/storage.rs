use gfx;
use nes::{ErrorInfo,ErrorInfoTrait};
use gfx_gl;

use types::*;

use object_pool::growable::Pool;

//use gfx::traits::FactoryExt;

use render::Error;
use render::pipelines::{TerrainPSO, create_terrain_pso};

use super::texture::*;
use storage::TextureID;

pub trait TextureStorage<ID:TextureID,IB> {
    fn load_texture(&mut self, texture_buffer:IB, texture_id:ID) -> Result<(), Error>;
    fn delete_texture(&mut self, texture_id:ID) -> Result<(), Error>;
}

pub struct Storage {
    pub gfx_factory: gfx_gl::Factory,
    pub terrain_pso: TerrainPSO,
    //pub fake_texture = u32;

    pub textures_rgb:Pool<RgbTexture, RgbTexture>,
}

impl Storage {
    pub fn new(mut gfx_factory: gfx_gl::Factory) -> Result<Self,Error> {
        let terrain_pso=create_terrain_pso(&mut gfx_factory)?;
        //let fake_texture = load_texture_raw(&mut gfx_factory, Size2{w: 2, h: 2}, &[0; 4]);

        let storage=Storage {
            gfx_factory,
            terrain_pso,
            //fake_texture
            textures_rgb:Pool::new(),
        };

        ok!(storage)
    }
}

impl TextureStorage<RgbTextureID, RgbImage> for Storage {
    fn load_texture(&mut self, texture_buffer:RgbImage, texture_id:RgbTextureID) -> Result<(), Error> {
        let texture=RgbTexture::new(texture_buffer, &mut self.gfx_factory)?;

        let id=self.textures_rgb.insert(texture);
        assert_eq!(id, texture_id.get_id());

        ok!()
    }

    fn delete_texture(&mut self, texture_id:RgbTextureID) -> Result<(), Error> {
        ok!()
    }
}