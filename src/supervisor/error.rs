use std;
use nes::{ErrorInfo,ErrorInfoTrait};
use gfx;
use glutin;
use reactor;

use types::ThreadSource;
define_error!( Error,
    ThreadCrash(thread:ThreadSource) =>
        "[Supervisor] {1} has crashed",

    BrockenChannel(error:Box<reactor::BrockenChannel<ThreadSource>>) =>
        "{}",
    Poisoned() =>
        "Mutex has been poisoned"
);