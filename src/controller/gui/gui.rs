use std;
use glutin;

use glutin::ElementState;
use glutin::MouseButton;
use glutin::VirtualKeyCode;

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

    pub fn on_mouse_button(&mut self, button: MouseButton, state: ElementState) {
        self.input.on_mouse_button(button, state);
    }

    pub fn on_key(&mut self, key:VirtualKeyCode, state:ElementState){
        self.input.on_key(key, state);
    }
}