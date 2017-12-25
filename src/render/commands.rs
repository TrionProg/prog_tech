
use types::*;

use supervisor::SupervisorSender;
use controller::ControllerSender;
use process::ProcessSender;

use render::storage::{ObjectMesh,TerrainMesh};
use render::pipelines::ObjectVertex;
use::Camera;

use process::Tile;

pub enum RenderCommand {
    ThreadCrash(ThreadSource),

    SupervisorSender(SupervisorSender),
    ControllerSender(ControllerSender),
    ProcessSender(ProcessSender),
    Camera(Camera),

    SupervisorReady,
    SupervisorFinished,

    Tick,
    Shutdown,

    ResizeWindow(u32,u32),

    LoadTexture(LoadTexture),
    LoadMesh(LoadMesh),
    LoadLod(LoadLod),
    SetSlot(SetSlot),

    ResourcesReady,
    CreateMap,
    LoadTile(usize, usize, Tile),

    MoveCursor(u32,u32),
    SetCursorA(Option<(u32,u32)>),
    SetCursorB(Option<(u32,u32)>),
}

pub enum LoadTexture {
    RGBA(RgbaImage, RgbaTextureID),
}

impl Into<RenderCommand> for LoadTexture {
    fn into(self) -> RenderCommand {
        RenderCommand::LoadTexture(self)
    }
}

pub enum LoadMesh {
    Object(ObjectMesh, ObjectMeshID),
    Terrain(TerrainMesh, TerrainMeshID),
}

impl Into<RenderCommand> for LoadMesh {
    fn into(self) -> RenderCommand {
        RenderCommand::LoadMesh(self)
    }
}

pub enum LoadLod {
    Object(Vec<ObjectVertex>, ObjectLodID)
}

impl Into<RenderCommand> for LoadLod {
    fn into(self) -> RenderCommand {
        RenderCommand::LoadLod(self)
    }
}

pub enum SetSlot {
    Cursor(ObjectMeshID),
    CursorA(ObjectMeshID),
    CursorB(ObjectMeshID),
    Tile(ObjectMeshID),
    TerrainTexture(usize, RgbaTextureID),
    FloorMesh(TerrainMeshID),
    WallMesh(usize,TerrainMeshID),
    HoleMesh(usize,TerrainMeshID),
}

impl Into<RenderCommand> for SetSlot {
    fn into(self) -> RenderCommand {
        RenderCommand::SetSlot(self)
    }
}