use glutin::{Event};
//use context::{Context};
//use types::{Time};

pub enum ScreenCommand {
    PopScreen,
    PopPopup,
    PushScreen(Box<Screen>),
    PushPopup(Box<Screen>),
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum EventStatus {
    Handled,
    NotHandled,
}

pub trait Screen {
    //fn frame(&mut self, context: &mut Context, dtime: Time);
    //fn handle_event(&mut self, context: &mut Context, event: &Event) -> EventStatus;
}
