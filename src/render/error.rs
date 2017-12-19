use std;
use nes::{ErrorInfo,ErrorInfoTrait};
use gfx;
use glutin;
use reactor;

use types::ThreadSource;

define_error!( Error,
    ThreadCrash(thread:ThreadSource) =>
        "[Render] {1} has crashed",

    BrockenChannel(error:Box<reactor::BrockenChannel<ThreadSource>>) =>
        "{}",
    Poisoned() =>
        "Mutex has been poisoned",

    SwapBuffersError(swap_error:Box<glutin::ContextError>) =>
        "Swap buffers error:{1}",
    CompileShaderError(compile_error:Box<gfx::shade::ProgramError>) =>
        "Compile shader error:{1}",
    CreatePSOError(pso_error:Box<String>) =>
        "Create PSO error:{1}",
    CreateTextureError(texture_error:Box<gfx::CombinedError>) =>
        "Create Texture Error: {1}",
    NoTexture() => //TODO
        "Texture does not exists",
    NoMesh() => //TODO
        "Mesh does not exists",
    NoLod() => //TODO
        "Lod does not exists",
    Other(message:String) =>
        "{}"
);

use camera::Error as CameraError;
impl From<CameraError> for Error{
    fn from(camera_error:CameraError) -> Self {
        match camera_error {
            CameraError::Poisoned(error_info) => Error::Poisoned(error_info)
        }
    }
}