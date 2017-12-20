
use types::{RgbaTextureID,TerrainMeshID};

use object_pool::growable::ID;
use storage::{TextureID,MeshID};

pub struct Slots {
    pub terrain_textures:Vec<RgbaTextureID>,
    pub floor_mesh:TerrainMeshID,
    pub wall_meshes:Vec<TerrainMeshID>,
    pub hole_meshes:Vec<TerrainMeshID>
}

impl Slots {
    pub fn new() -> Self {
        let terrain_textures=vec![RgbaTextureID::new(ID::zeroed());5];
        let wall_meshes=vec![TerrainMeshID::new(ID::zeroed());16];
        let hole_meshes=vec![TerrainMeshID::new(ID::zeroed());16];

        let slots=Slots {
            terrain_textures,
            floor_mesh:TerrainMeshID::new(ID::zeroed()),
            wall_meshes,
            hole_meshes
        };

        slots
    }
}