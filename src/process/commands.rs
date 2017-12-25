use glutin;

use types::*;

use glutin::EventsLoop;

use supervisor::SupervisorSender;
use render::RenderSender;
use controller::ControllerSender;

pub enum ProcessCommand {
    ThreadCrash(ThreadSource),

    SupervisorSender(SupervisorSender),
    RenderSender(RenderSender),
    ControllerSender(ControllerSender),

    EventsLoop(EventsLoop),

    SupervisorReady,
    SupervisorFinished,

    Tick,
    Shutdown,

    ResourcesLoaded,
    Algorithm((u32,u32),(u32,u32))
}