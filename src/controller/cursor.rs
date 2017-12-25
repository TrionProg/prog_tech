use nes::{ErrorInfo,ErrorInfoTrait};

use consts::MAP_SIZE;

use glutin::ElementState;

use render::{RenderSender,RenderCommand};
use process::{ProcessSender,ProcessCommand};
use super::Error;

pub struct Cursor {
    render_sender:RenderSender,
    process_sender:ProcessSender,

    pub x:u32,
    pub z:u32,
    pub a:Option<(u32,u32)>,
    pub b:Option<(u32,u32)>,

    left_prescaler:u32,
    right_prescaler:u32,
    back_prescaler:u32,
    front_prescaler:u32,
}

impl Cursor {
    pub fn new(render_sender:RenderSender, process_sender:ProcessSender) -> Self {
        Cursor {
            render_sender,
            process_sender,

            x:0,
            z:0,
            a:None,
            b:None,

            left_prescaler:0,
            right_prescaler:0,
            back_prescaler:0,
            front_prescaler:0,
        }
    }

    pub fn move_left(&mut self, state:ElementState) -> bool {
        match state {
            ElementState::Pressed => {
                if self.left_prescaler==0 {
                    self.left_prescaler=5;

                    if self.x>0 {
                        self.x-=1;
                        true
                    }else{
                        false
                    }
                }else{
                    self.left_prescaler-=1;
                    false
                }
            },
            _ => {
                self.left_prescaler=0;
                false
            }
        }
    }

    pub fn move_right(&mut self, state:ElementState) -> bool {
        match state {
            ElementState::Pressed => {
                if self.right_prescaler==0 {
                    self.right_prescaler=5;

                    if self.x<(MAP_SIZE-2) as u32 {
                        self.x+=1;
                        true
                    }else{
                        false
                    }
                }else{
                    self.right_prescaler-=1;
                    false
                }
            },
            _ => {
                self.right_prescaler=0;
                false
            }
        }
    }

    pub fn move_back(&mut self, state:ElementState) -> bool {
        match state {
            ElementState::Pressed => {
                if self.back_prescaler==0 {
                    self.back_prescaler=5;

                    if self.z>0 {
                        self.z-=1;
                        true
                    }else{
                        false
                    }
                }else{
                    self.back_prescaler-=1;
                    false
                }
            },
            _ => {
                self.back_prescaler=0;
                false
            }
        }
    }

    pub fn move_front(&mut self, state:ElementState) -> bool {
        match state {
            ElementState::Pressed => {
                if self.front_prescaler==0 {
                    self.front_prescaler=5;

                    if self.z<(MAP_SIZE-2) as u32 {
                        self.z+=1;
                        true
                    }else{
                        false
                    }
                }else{
                    self.front_prescaler-=1;
                    false
                }
            },
            _ => {
                self.front_prescaler=0;
                false
            }
        }
    }

    pub fn on_enter(&mut self) -> Result<(),Error> {
        if self.a.is_none() {
            self.a=Some((self.x,self.z));
            try_send!(self.render_sender, RenderCommand::SetCursorA( self.a ));
            return ok!();
        }

        if self.b.is_none() {
            self.b=Some((self.x,self.z));
            try_send!(self.render_sender, RenderCommand::SetCursorB( self.b ));

            try_send!(self.process_sender, ProcessCommand::Algorithm( self.a.unwrap(), self.b.unwrap() ));
        }

        ok!()
    }

    pub fn algorithm_end(&mut self) -> Result<(),Error> {
        //self.a=None;
        self.b=None;

        //try_send!(self.render_sender, RenderCommand::SetCursorA( self.a ));
        try_send!(self.render_sender, RenderCommand::SetCursorB( self.b ));

        ok!()
    }
}