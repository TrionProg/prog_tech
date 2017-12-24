
use types::{RgbaTextureID,TerrainMeshID,ObjectMeshID};

use object_pool::growable::ID;
use storage::{TextureID,MeshID};

pub struct Slots {
    //pub loading_texture:RgbaTextureID,
    //pub loading_mesh:TerrainMeshID,

    pub cursor:ObjectMeshID,
    pub tile:ObjectMeshID,
    pub terrain_textures:Vec<RgbaTextureID>,
    pub floor_mesh:TerrainMeshID,
    pub wall_meshes:Vec<TerrainMeshID>,
    pub hole_meshes:Vec<TerrainMeshID>
}

impl Slots {
    pub fn new() -> Self {
        let terrain_textures=vec![RgbaTextureID::zeroed();5];
        let wall_meshes=vec![TerrainMeshID::zeroed();16];
        let hole_meshes=vec![TerrainMeshID::zeroed();16];

        let slots=Slots {
            cursor:ObjectMeshID::zeroed(),
            tile:ObjectMeshID::zeroed(),
            terrain_textures,
            floor_mesh:TerrainMeshID::new(ID::zeroed()),
            wall_meshes,
            hole_meshes
        };

        slots
    }
}