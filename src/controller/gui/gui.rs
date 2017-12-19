use std;
use glutin;

use glutin::ElementState;
use glutin::MouseButton;

use super::Input;

pub struct GUI{
    pub input:Input,
}

impl GUI {
    pub fn new() -> Self {
        let gui = GUI {
            input: Input::new(),
        };

        gui
    }

    pub fn on_mouse_move(&mut self, x: i32, y: i32) {
        self.input.on_mouse_move(x, y);
    }

    pub fn on_mouse_button(&mut self, state: ElementState, button: MouseButton) {
        self.input.on_mouse_button(state, button);
    }
}