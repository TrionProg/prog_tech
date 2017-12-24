use glutin;
use glutin::Event;
use glutin::ElementState;
use glutin::MouseButton;
use glutin::VirtualKeyCode;

use consts::*;

pub struct Input{
    pub mouse_x:Option<i32>,
    pub mouse_y:Option<i32>,
    pub mouse_move_x:i32,
    pub mouse_move_y:i32,

    pub left_mouse_button:ElementState,
    pub middle_mouse_button:ElementState,
    pub right_mouse_button:ElementState,

    pub key_states:[ElementState;KEY_LIMIT],
}

impl Input{
    pub fn new() -> Self{
        Input{
            mouse_x:None,
            mouse_y:None,
            mouse_move_x:0,
            mouse_move_y:0,

            left_mouse_button:ElementState::Released,
            middle_mouse_button:ElementState::Released,
            right_mouse_button:ElementState::Released,

            key_states:[ElementState::Released;KEY_LIMIT],
        }
    }

    pub fn on_mouse_move(&mut self, x:i32, y:i32){
        self.mouse_move_x=match self.mouse_x{
            Some( ref mouse_x ) => x-mouse_x,
            None => 0,
        };

        self.mouse_move_y=match self.mouse_y{
            Some( ref mouse_y ) => y-mouse_y,
            None => 0,
        };

        self.mouse_x = Some(x);
        self.mouse_y = Some(y);
    }

    pub fn on_mouse_button(&mut self, button:MouseButton, state:ElementState){
        match button{
            MouseButton::Left => self.left_mouse_button=state,
            MouseButton::Middle => self.middle_mouse_button=state,
            MouseButton::Right => self.right_mouse_button=state,
            MouseButton::Other(_) => {},
        }
    }

    pub fn on_key(&mut self, key:VirtualKeyCode, state:ElementState){
        self.key_states[key as usize]=state;
    }

    pub fn key(&self, key:VirtualKeyCode) -> ElementState {
        self.key_states[key as usize]
    }
}

/*
        match event {
            &glutin::Event::KeyboardInput(glutin::ElementState::Pressed, _, Some(glutin::VirtualKeyCode::Space)) => {
                self.moving_up = true;
            },
            &glutin::Event::KeyboardInput(glutin::ElementState::Released, _, Some(glutin::VirtualKeyCode::Space)) => {
                self.moving_up = false;
            },
            &glutin::Event::KeyboardInput(glutin::ElementState::Pressed, _, Some(glutin::VirtualKeyCode::Down)) => {
                self.moving_down = true;
            },
            &glutin::Event::KeyboardInput(glutin::ElementState::Released, _, Some(glutin::VirtualKeyCode::Down)) => {
                self.moving_down = false;
            },
            &glutin::Event::KeyboardInput(glutin::ElementState::Pressed, _, Some(glutin::VirtualKeyCode::A)) => {
                self.moving_left = true;
            },
            &glutin::Event::KeyboardInput(glutin::ElementState::Released, _, Some(glutin::VirtualKeyCode::A)) => {
                self.moving_left = false;
            },
            &glutin::Event::KeyboardInput(glutin::ElementState::Pressed, _, Some(glutin::VirtualKeyCode::D)) => {
                self.moving_right = true;
            },
            &glutin::Event::KeyboardInput(glutin::ElementState::Released, _, Some(glutin::VirtualKeyCode::D)) => {
                self.moving_right = false;
            },
            &glutin::Event::KeyboardInput(glutin::ElementState::Pressed, _, Some(glutin::VirtualKeyCode::W)) => {
                self.moving_forward = true;
            },
            &glutin::Event::KeyboardInput(glutin::ElementState::Released, _, Some(glutin::VirtualKeyCode::W)) => {
                self.moving_forward = false;
            },
            &glutin::Event::KeyboardInput(glutin::ElementState::Pressed, _, Some(glutin::VirtualKeyCode::S)) => {
                self.moving_backward = true;
            },
            &glutin::Event::KeyboardInput(glutin::ElementState::Released, _, Some(glutin::VirtualKeyCode::S)) => {
                self.moving_backward = false;
            },
            _ => {}
        }
    }
*/
