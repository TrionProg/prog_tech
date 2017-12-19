use std;
use nes::{ErrorInfo,ErrorInfoTrait};
use gfx;
use glutin;
use reactor;

use types::ThreadSource;
define_error!( Error,
    ThreadCrash(thread:ThreadSource) =>
        "[Controller] {1} has crashed",

    BrockenChannel(error:Box<reactor::BrockenChannel<ThreadSource>>) =>
        "{}",
    Poisoned() =>
        "Mutex has been poisoned"
);

use camera::Error as CameraError;
impl From<CameraError> for Error{
    fn from(camera_error:CameraError) -> Self {
        match camera_error {
            CameraError::Poisoned(error_info) => Error::Poisoned(error_info)
        }
    }
}