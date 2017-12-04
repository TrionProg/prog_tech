
use types::*;

use storage::Storage;

use process::ProcessSender;

pub enum RenderCommand {
    ProcessSender(ProcessSender),
    Storage(Storage),
    ProcessSetupError,

    LoadTexture(LoadTexture),
}

pub enum LoadTexture {
    RGB(RgbImage, RgbaTextureID),
}

impl Into<RenderCommand> for LoadTexture {
    fn into(self) -> RenderCommand {
        RenderCommand::LoadTexture(self)
    }
}