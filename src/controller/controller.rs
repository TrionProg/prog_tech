use std;
use glutin;
use nes::{ErrorInfo,ErrorInfoTrait};
use reactor;

use types::*;

use glutin::EventsLoop;
use glutin::WindowEvent;
use glutin::ElementState;
use glutin::VirtualKeyCode;

use std::thread;
use std::thread::JoinHandle;

use supervisor;
use supervisor::SupervisorSender;
use supervisor::SupervisorCommand;

use render;
use render::RenderSender;
use render::RenderCommand;

use process;
use process::ProcessSender;
use process::ProcessCommand;

use ::Camera;

use super::Error;
use super::ControllerCommand;
use super::GUI;
use super::Cursor;

pub type ControllerSender = reactor::Sender<ThreadSource,ControllerCommand>;
pub type ControllerReceiver = reactor::Receiver<ThreadSource,ControllerCommand>;

pub struct Controller {
    controller_receiver:ControllerReceiver,
    supervisor_sender:SupervisorSender,
    render_sender:RenderSender,
    process_sender:ProcessSender,

    events_loop:EventsLoop,
    gui:GUI,
    camera:Camera,
    cursor:Cursor
}

impl Controller{
    pub fn run()-> (JoinHandle<()>, ControllerSender) {
        let (controller_sender, mut controller_receiver) = reactor::create_channel(ThreadSource::Controller);

        let join_handle=thread::Builder::new().name("Controller".to_string()).spawn(move|| {
            let (mut supervisor_sender, mut render_sender, mut process_sender) = Self::get_senders(&mut controller_receiver).unwrap();

            println!("C1");

            let camera=Camera::new(1024, 768);

            send![
                render_sender, RenderCommand::Camera(camera.clone())
            ].unwrap();

            let events_loop=wait![controller_receiver,
                ControllerCommand::EventsLoop(events_loop) => events_loop
            ].unwrap();

            println!("C2");

            let mut controller=match Self::setup(
                controller_receiver,
                supervisor_sender.clone(),
                render_sender.clone(),
                process_sender.clone(),
                events_loop,
                camera
            ) {
                Ok(controller) => controller,
                Err(error) => {
                    println!("Controller setup error: {}", error);

                    send![
                        supervisor_sender, SupervisorCommand::ThreadCrash(ThreadSource::Controller),
                        render_sender, RenderCommand::ThreadCrash(ThreadSource::Controller),
                        process_sender, ProcessCommand::ThreadCrash(ThreadSource::Controller)
                    ].unwrap();

                    return;
                }
            };

            println!("C3");

            controller.synchronize_setup().unwrap();

            println!("C4");

            match controller.lifecycle() {
                Ok(_) => {
                    //do something

                    println!("C5");

                    controller.synchronize_finish().unwrap();
                }
                Err(error) => {
                    println!("Controller Error: {}!", error);

                    match error {//TODO BrockenChannel
                        Error::ThreadCrash(_,thread) => {
                            /*
                            if source==ThreadSource::Disk {
                                try_send![disk.storage_sender, StorageCommand::IpcListenerThreadCrash(source)];
                            }
                            */
                        },
                        _ => {
                            send![
                                controller.supervisor_sender, SupervisorCommand::ThreadCrash(ThreadSource::Controller),
                                controller.render_sender, RenderCommand::ThreadCrash(ThreadSource::Controller),
                                controller.process_sender, ProcessCommand::ThreadCrash(ThreadSource::Controller)
                            ].unwrap();
                        }
                    }
                }
            }

            println!("C6");
        }).unwrap();

        (join_handle, controller_sender)
    }

    fn get_senders(receiver:&mut ControllerReceiver) -> Result<(SupervisorSender,RenderSender,ProcessSender),Error> {
        let supervisor_sender=wait![receiver,
            ControllerCommand::SupervisorSender(supervisor_sender) => supervisor_sender
        ].unwrap();

        let render_sender=wait![receiver,
            ControllerCommand::RenderSender(render_sender) => render_sender
        ].unwrap();

        let process_sender=wait![receiver,
            ControllerCommand::ProcessSender(process_sender) => process_sender
        ].unwrap();

        ok!((supervisor_sender,render_sender,process_sender))
    }

    fn setup(
        controller_receiver:ControllerReceiver,
        supervisor_sender:SupervisorSender,
        render_sender:RenderSender,
        process_sender:ProcessSender,
        events_loop:EventsLoop,
        camera:Camera,
    ) -> Result<Self,Error> {
        let cursor=Cursor::new(render_sender.clone(),process_sender.clone());

        let controller=Controller {
            controller_receiver,
            supervisor_sender,
            render_sender,
            process_sender,

            events_loop,
            gui:GUI::new(),
            camera,
            cursor
        };

        ok!(controller)
    }

    fn synchronize_setup(&mut self) -> Result<(),Error>{
        try_send![self.supervisor_sender, SupervisorCommand::ThreadReady(ThreadSource::Controller)];

        wait![self.controller_receiver,
            ControllerCommand::SupervisorReady => ()
        ].unwrap();

        ok!()
    }

    fn lifecycle(&mut self) -> Result<(),Error> {
        loop {
            self.poll_window_events()?;

            if self.handle_controller_commands()? {
                println!("QUIT3");
                return ok!();
            }

            self.handle_cursor()?;
        }
    }


    fn poll_window_events(&mut self) -> Result<(),Error> {
        let mut quit=false;

        let events_loop=&mut self.events_loop;
        let gui=&mut self.gui;
        let cursor=&mut self.cursor;
        let camera=&self.camera;
        let supervisor_sender=&mut self.supervisor_sender;
        let render_sender=&mut self.render_sender;
        let mut result=Ok(());

        events_loop.poll_events(move|event| {
            if let glutin::Event::WindowEvent { event, .. } = event {

                let mut handle_event=||{
                    match event {
                        WindowEvent::KeyboardInput {
                            input: glutin::KeyboardInput {
                                virtual_keycode: Some(glutin::VirtualKeyCode::Escape),
                                .. },
                            ..
                        } | WindowEvent::Closed =>
                            try_send!(supervisor_sender, SupervisorCommand::Quit),
                        WindowEvent::Resized(width, height) => {
                            camera.resize(width,height)?;
                            try_send!(render_sender, RenderCommand::ResizeWindow(width,height));
                        },
                        WindowEvent::MouseMoved {device_id, position: (x, y)} => {
                            gui.on_mouse_move(x as i32,y as i32);

                            if gui.input.left_mouse_button==ElementState::Pressed {
                                camera.rotate(&gui.input)?;
                            }
                        },
                        WindowEvent::MouseInput{device_id, state, button} => {
                            gui.on_mouse_button(button, state);

                            if gui.input.right_mouse_button==ElementState::Pressed {

                            }
                        },
                        WindowEvent::MouseWheel {device_id, delta, phase} => {
                            camera.on_mouse_wheel(delta)?;
                        },
                        WindowEvent::KeyboardInput {device_id, input} => {
                            match input.virtual_keycode {
                                Some(key) => {
                                    gui.on_key(key, input.state);

                                    if key==VirtualKeyCode::Return && input.state==ElementState::Released {
                                        cursor.on_enter()?;
                                    }
                                }
                                _ => {},
                            }
                        },
                        _ => {},
                    }

                    ok!(())
                };

                result=handle_event();

                if result.is_err() {
                    return;
                }
            }
        });

        ok!()
    }

    fn handle_cursor(&mut self) -> Result<(),Error> {
        let mut moved = false;
        moved |= self.cursor.move_left(self.gui.input.key(VirtualKeyCode::Left));
        moved |= self.cursor.move_right(self.gui.input.key(VirtualKeyCode::Right));
        moved |= self.cursor.move_back(self.gui.input.key(VirtualKeyCode::Up));
        moved |= self.cursor.move_front(self.gui.input.key(VirtualKeyCode::Down));

        if moved {
            try_send!(self.render_sender, RenderCommand::MoveCursor(self.cursor.x,self.cursor.z));
        }

        ok!()
    }

    fn handle_controller_commands(&mut self) -> Result<bool,Error> {
        loop {
            match try_recv_block!(self.controller_receiver) {
                ControllerCommand::ThreadCrash(thread) => return err!(Error::ThreadCrash, thread),
                ControllerCommand::Tick => return ok!(false),
                ControllerCommand::Shutdown => return ok!(true),

                ControllerCommand::AlgorithmEnd =>
                    self.cursor.algorithm_end()?,
                _ => unreachable!()
            }
        }
    }

    fn synchronize_finish(&mut self) -> Result<(),Error>{
        println!("C F1");
        try_send![self.supervisor_sender, SupervisorCommand::ThreadFinished(ThreadSource::Controller)];
        println!("C F2");

        wait![self.controller_receiver,
            ControllerCommand::SupervisorFinished => ()
        ].unwrap();

        println!("C F");

        ok!()
    }
}