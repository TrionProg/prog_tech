use glutin;

use types::*;

use glutin::EventsLoop;

use supervisor::SupervisorSender;
use render::RenderSender;

pub enum ControllerCommand {
    ThreadCrash(ThreadSource),

    SupervisorSender(SupervisorSender),
    RenderSender(RenderSender),

    EventsLoop(EventsLoop),

    SupervisorReady,
    SupervisorFinished,

    Tick,
    Shutdown,
}