use std;
use nes::{ErrorInfo,ErrorInfoTrait};
use gfx;
use glutin;
use reactor;
use storage;

use types::ThreadSource;
define_error!( Error,
    ThreadCrash(thread:ThreadSource) =>
        "[Process] {1} has crashed",

    BrockenChannel(error:Box<reactor::BrockenChannel<ThreadSource>>) =>
        "{}",
    Poisoned() =>
        "Mutex has been poisoned",

    StorageError(storage_error:Box<storage::Error>) =>
        "Storage error:{}"
);


//TODO
impl_from_error!(storage::Error => Error::StorageError);