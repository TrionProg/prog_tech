use std;
use nes::{ErrorInfo,ErrorInfoTrait};

use types::ThreadSource;
define_error!( Error,
    Poisoned() =>
        "Mutex has been poisoned"
);