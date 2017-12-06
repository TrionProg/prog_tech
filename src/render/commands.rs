
use types::*;

use process::ProcessSender;

use render::storage::ObjectMesh;
use render::pipelines::ObjectVertex;

pub enum RenderCommand {
    ProcessThreadCrash(ThreadSource),

    ProcessSender(ProcessSender),
    ProcessSetupError,

    ProcessIsReady,
    ProcessFinished,
    Shutdown,
}

pub enum StorageCommand {
    LoadTexture(LoadTexture),
    LoadMesh(LoadMesh),
    LoadLod(LoadLod),
}

pub enum LoadTexture {
    RGBA(RgbaImage, RgbaTextureID),
}

impl Into<StorageCommand> for LoadTexture {
    fn into(self) -> StorageCommand {
        StorageCommand::LoadTexture(self)
    }
}

pub enum LoadMesh {
    Object(ObjectMesh, ObjectMeshID),
}

impl Into<StorageCommand> for LoadMesh {
    fn into(self) -> StorageCommand {
        StorageCommand::LoadMesh(self)
    }
}

pub enum LoadLod {
    Object(Vec<ObjectVertex>, ObjectLodID)
}

impl Into<StorageCommand> for LoadLod {
    fn into(self) -> StorageCommand {
        StorageCommand::LoadLod(self)
    }
}