use glutin;

use types::*;

use glutin::EventsLoop;

use supervisor::SupervisorSender;
use render::RenderSender;
use process::ProcessSender;

pub enum ControllerCommand {
    ThreadCrash(ThreadSource),

    SupervisorSender(SupervisorSender),
    RenderSender(RenderSender),
    ProcessSender(ProcessSender),

    EventsLoop(EventsLoop),

    SupervisorReady,
    SupervisorFinished,

    Tick,
    Shutdown,
    AlgorithmEnd,
}