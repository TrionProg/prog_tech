use std;
use nes::{ErrorInfo,ErrorInfoTrait};
use reactor;

use types::ThreadSource;

define_error!( Error,
    BrockenChannel(error:Box<reactor::BrockenChannel<ThreadSource>>) =>
        "{}",
    Poisoned() =>
        "Mutex has been poisoned",


    OpenImageFileError(file_name:String) =>
        "Can not open image \"{}\"",
    ReadImageFileError(file_name:String) =>
        "Can not read image \"{}\""
);