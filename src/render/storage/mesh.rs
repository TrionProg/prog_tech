use gfx;
use gfx_gl;

use types::*;

use gfx::Factory;
use gfx::traits::FactoryExt;

pub trait Mesh {
}

pub struct ObjectMesh {
    pub lod:ObjectLodID,
    pub texture:RgbaTextureID,
}

impl ObjectMesh {
    pub fn new(lod:ObjectLodID, texture:RgbaTextureID) -> Self {
        ObjectMesh {
            lod,
            texture
        }
    }
}

impl Mesh for ObjectMesh{}



pub struct TerrainMesh {
    pub lod:ObjectLodID,
}

impl TerrainMesh {
    pub fn new(lod:ObjectLodID) -> Self {
        TerrainMesh {
            lod
        }
    }
}

impl Mesh for TerrainMesh{}