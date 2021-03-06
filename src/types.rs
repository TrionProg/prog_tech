use std;
use object_pool::growable::ID;

pub use std::time::SystemTime as Time;
pub use std::time::Duration;

pub type TextureData=Vec<u8>;

pub use image::{GrayImage, GrayAlphaImage, RgbImage, RgbaImage};
pub use storage::{RgbaTextureID};
pub use storage::{ObjectMeshID, TerrainMeshID, TraceMeshID};
pub use storage::{ObjectLodID, TraceLodID};

#[derive(Debug,Copy,Clone,Eq,PartialEq)]
pub enum ThreadSource{
    Supervisor=0,
    Render=1,
    Process=2,
    Controller=3,
    Algorithm=4
}

impl std::fmt::Display for ThreadSource{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self{
            ThreadSource::Supervisor => write!(f, "Supervisor"),
            ThreadSource::Render => write!(f, "Render"),
            ThreadSource::Process => write!(f, "Process"),
            ThreadSource::Controller => write!(f, "Controller"),
            ThreadSource::Algorithm => write!(f, "Algorithm"),
        }
    }
}

impl ::reactor::ThreadTrait for ThreadSource{}

pub struct TraceID(ID);

impl TraceID {
    pub fn new(id:ID) -> Self {TraceID(id)}
    pub fn zeroed() -> Self {TraceID(ID::zeroed())}
    pub fn get_id(&self) -> ID {self.0}
}
/*
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum MeshID {
    Terrain(ID)
}

*/