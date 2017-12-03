use std;
use nes::{ErrorInfo,ErrorInfoTrait};
use gfx;
use glutin;

//use ::ThreadSource;

define_error!( Error,
    BrockenChannel() =>
        "Broken channel",
    Poisoned() =>
        "Mutex has been poisoned",

    SwapBuffersError(swap_error:Box<glutin::ContextError>) =>
        "Swap buffers error:{1}",
    CompileShaderError(compile_error:Box<gfx::shade::ProgramError>) =>
        "Compile shader error:{1}",
    CreatePSOError(pso_error:Box<String>) =>
        "Create PSO error:{1}",
    Other(message:String) =>
        "{}"
);