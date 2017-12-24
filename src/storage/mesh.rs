
use object_pool::growable::ID;

pub trait MeshID {
    fn new(id:ID) -> Self;
    fn zeroed() -> Self;
    fn get_id(&self) -> ID;
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct ObjectMeshID(ID);

impl MeshID for ObjectMeshID {
    fn new(id:ID) -> Self {ObjectMeshID(id)}
    fn zeroed() -> Self {ObjectMeshID(ID::zeroed())}
    fn get_id(&self) -> ID {self.0}
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct TerrainMeshID(ID);

impl MeshID for TerrainMeshID {
    fn new(id:ID) -> Self {TerrainMeshID(id)}
    fn zeroed() -> Self {TerrainMeshID(ID::zeroed())}
    fn get_id(&self) -> ID {self.0}
}