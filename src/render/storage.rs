use gfx;
use nes::{ErrorInfo,ErrorInfoTrait};
use gfx_gl;

//use gfx::traits::FactoryExt;

use super::Error;
use super::pipelines::{TerrainPSO, create_terrain_pso};

pub struct Storage {
    pub gfx_factory: gfx_gl::Factory,
    pub terrain_pso: TerrainPSO,
    //pub fake_texture = u32;
}

impl Storage {
    pub fn new(mut gfx_factory: gfx_gl::Factory) -> Result<Self,Error> {
        let terrain_pso=create_terrain_pso(&mut gfx_factory)?;
        //let fake_texture = load_texture_raw(&mut gfx_factory, Size2{w: 2, h: 2}, &[0; 4]);

        let storage=Storage {
            gfx_factory,
            terrain_pso,
            //fake_texture
        };

        ok!(storage)
    }
}