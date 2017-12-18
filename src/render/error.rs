use std;
use nes::{ErrorInfo,ErrorInfoTrait};
use reactor;
use gfx;
use glutin;

use types::ThreadSource;

//use ::ThreadSource;

define_error!( Error,
    RenderThreadCrash(thread_source:ThreadSource) =>
        "[Source:{1}] Render thread has finished incorrecty(crashed)",
    ProcessThreadCrash(thread_source:ThreadSource) =>
        "[Source:{1}] Process thread has finished incorrecty(crashed)",

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