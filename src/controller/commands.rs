
use types::*;

pub enum ControllerCommand {
    ProcessThreadCrash(ThreadSource),

    ProcessSender(ProcessSender),
    ProcessSetupError,

    ProcessIsReady,
    ProcessFinished,
    Shutdown,
}