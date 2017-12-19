use glutin;

use types::*;

#[derive(Debug)]
pub enum SupervisorCommand {
    ThreadCrash(ThreadSource),

    ThreadReady(ThreadSource),
    ThreadFinished(ThreadSource),

    Quit,
}