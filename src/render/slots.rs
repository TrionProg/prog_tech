
use types::{RgbaTextureID,TerrainMeshID,ObjectMeshID};

use object_pool::growable::ID;
use storage::{TextureID,MeshID};

use render::SetSlot;

pub struct Slots {
    //pub loading_texture:RgbaTextureID,
    //pub loading_mesh:TerrainMeshID,

    pub cursor:ObjectMeshID,
    pub cursor_a:ObjectMeshID,
    pub cursor_b:ObjectMeshID,
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
            cursor_a:ObjectMeshID::zeroed(),
            cursor_b:ObjectMeshID::zeroed(),
            tile:ObjectMeshID::zeroed(),
            terrain_textures,
            floor_mesh:TerrainMeshID::new(ID::zeroed()),
            wall_meshes,
            hole_meshes
        };

        slots
    }

    pub fn set_slot(&mut self, set_slot:SetSlot) {
        match set_slot {
            SetSlot::Cursor(mesh_id) =>
                self.cursor=mesh_id,
            SetSlot::CursorA(mesh_id) =>
                self.cursor_a=mesh_id,
            SetSlot::CursorB(mesh_id) =>
                self.cursor_b=mesh_id,
            SetSlot::Tile(mesh_id) =>
                self.tile=mesh_id,
            SetSlot::TerrainTexture(index, texture_id) =>
                self.terrain_textures[index]=texture_id,
            SetSlot::FloorMesh(mesh_id) =>
                self.floor_mesh=mesh_id,
            SetSlot::WallMesh(index, mesh_id) =>
                self.wall_meshes[index]=mesh_id,
            SetSlot::HoleMesh(index, mesh_id) =>
                self.hole_meshes[index]=mesh_id,
        }
    }
}