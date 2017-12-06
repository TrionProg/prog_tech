
use types::*;

pub enum ProcessCommand {
    RenderThreadCrash(ThreadSource),

    RenderSetupError,
    RenderIsReady,
    RenderFinished,

    Quit
}