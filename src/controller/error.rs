use std;
use nes::{ErrorInfo,ErrorInfoTrait};
use gfx;
use glutin;

use types::ThreadSource;

//use ::ThreadSource;

define_error!( Error,
    RenderThreadCrash(thread_source:ThreadSource) =>
        "[Source:{1}] Render thread has finished incorrecty(crashed)",
    ProcessThreadCrash(thread_source:ThreadSource) =>
        "[Source:{1}] Process thread has finished incorrecty(crashed)",

    BrockenChannel() =>
        "Broken channel",
    Poisoned() =>
        "Mutex has been poisoned"
);