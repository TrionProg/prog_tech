
use types::*;

pub enum ProcessCommand {
    RenderThreadCrash(ThreadSource),

    RenderSetupError,
    RenderIsReady,
    RenderFinished,

    ControllerSetupError,
    ControllerIsReady,
    ControllerFinished,

    Quit
}