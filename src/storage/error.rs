use std;
use nes::{ErrorInfo,ErrorInfoTrait};

//use ::ThreadSource;

define_error!( Error,
    BrockenChannel() =>
        "Broken channel",
    Poisoned() =>
        "Mutex has been poisoned"
);