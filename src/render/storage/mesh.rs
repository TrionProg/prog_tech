use gfx;
use gfx_gl;

use types::*;

use gfx::Factory;
use gfx::traits::FactoryExt;

pub trait Mesh {
}

pub struct ObjectMesh {
    pub lod:ObjectLodID,
    pub texture:RgbTextureID,
}

impl ObjectMesh {
    pub fn new(lod:ObjectLodID, texture:RgbTextureID) -> Self {
        ObjectMesh {
            lod,
            texture
        }
    }
}

impl Mesh for ObjectMesh{}