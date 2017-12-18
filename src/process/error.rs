use std;
use nes::{ErrorInfo,ErrorInfoTrait};
use reactor;

use types::*;

define_error!( Error,
    RenderThreadCrash(thread_source:ThreadSource) =>
        "[Source:{1}] Render thread has finished incorrecty(crashed)",
    ProcessThreadCrash(thread_source:ThreadSource) =>
        "[Source:{1}] Process thread has finished incorrecty(crashed)",

    BrockenChannel(error:Box<reactor::BrockenChannel<ThreadSource>>) =>
        "{}",
    Poisoned() =>
        "Mutex has been poisoned"
);