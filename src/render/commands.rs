
use types::*;

use supervisor::SupervisorSender;
use controller::ControllerSender;
use process::ProcessSender;

use render::storage::ObjectMesh;
use render::pipelines::ObjectVertex;
use::Camera;

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

    ResourcesReady,
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