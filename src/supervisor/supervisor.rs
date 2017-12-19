use std;
use nes::{ErrorInfo,ErrorInfoTrait};
use reactor;

use types::*;

use std::thread;
use std::thread::JoinHandle;


use render;
use render::Render;
use render::RenderSender;
use render::RenderCommand;

use controller;
use controller::Controller;
use controller::ControllerSender;
use controller::ControllerCommand;

use process;
use process::Process;
use process::ProcessSender;
use process::ProcessCommand;

use super::Error;
use super::SupervisorCommand;

pub type SupervisorSender = reactor::Sender<ThreadSource,SupervisorCommand>;
pub type SupervisorReceiver = reactor::Receiver<ThreadSource,SupervisorCommand>;

pub struct Supervisor {
    supervisor_receiver:SupervisorReceiver,
    supervisor_sender:SupervisorSender,
    render_sender:RenderSender,
    controller_sender:ControllerSender,
    process_sender:ProcessSender
}

impl Supervisor {
    pub fn run() {
        let (supervisor_sender, mut supervisor_receiver) = reactor::create_channel(ThreadSource::Render);
        let (render_join_handler, mut render_sender) = Render::run();
        let (controller_join_handler, mut controller_sender) = Controller::run();
        let (process_join_handler, mut process_sender) = Process::run();

        println!("S1");

        send![
            render_sender, RenderCommand::SupervisorSender(supervisor_sender.clone()),
            render_sender, RenderCommand::ControllerSender(controller_sender.clone()),
            render_sender, RenderCommand::ProcessSender(process_sender.clone())
        ].unwrap();

        send![
            controller_sender, ControllerCommand::SupervisorSender(supervisor_sender.clone()),
            controller_sender, ControllerCommand::RenderSender(render_sender.clone()),
            controller_sender, ControllerCommand::ProcessSender(process_sender.clone())
        ].unwrap();

        send![
            process_sender, ProcessCommand::SupervisorSender(supervisor_sender.clone()),
            process_sender, ProcessCommand::RenderSender(render_sender.clone()),
            process_sender, ProcessCommand::ControllerSender(controller_sender.clone())
        ].unwrap();

        println!("S2");

        let mut supervisor=Self::setup(
            supervisor_receiver,
            supervisor_sender.clone(),
            render_sender.clone(),
            controller_sender.clone(),
            process_sender.clone(),
        );

        println!("S3");

        supervisor.synchronize_setup().unwrap();

        println!("S4");

        match supervisor.lifecycle() {
            Ok(_) => {
                println!("S5");
                supervisor.synchronize_finish().unwrap();
            },
            Err(error) => {
                println!("Supervisor Error: {}!", error);

                match error {
                    Error::ThreadCrash(_,thread) => {
                        /*
                        if source==ThreadSource::Disk {
                            try_send![disk.storage_sender, StorageCommand::IpcListenerThreadCrash(source)];
                        }
                        */
                    }
                    _ => {
                        //TODO
                    }
                }
            }
        }

        controller_join_handler.join();
        render_join_handler.join();
        process_join_handler.join();
    }

    fn setup(
        supervisor_receiver:SupervisorReceiver,
        supervisor_sender:SupervisorSender,
        render_sender:RenderSender,
        controller_sender:ControllerSender,
        process_sender:ProcessSender
    ) -> Self {
        Supervisor {
            supervisor_receiver,
            supervisor_sender,
            render_sender,
            controller_sender,
            process_sender
        }
    }

    fn synchronize_setup(&mut self) -> Result<(),Error> {
        wait![self.supervisor_receiver,
            SupervisorCommand::ThreadReady(ThreadSource::Render) => ()
        ].unwrap();

        wait![self.supervisor_receiver,
            SupervisorCommand::ThreadReady(ThreadSource::Controller) => ()
        ].unwrap();

        wait![self.supervisor_receiver,
            SupervisorCommand::ThreadReady(ThreadSource::Process) => ()
        ].unwrap();

        send![
            self.render_sender, RenderCommand::SupervisorReady,
            self.controller_sender, ControllerCommand::SupervisorReady,
            self.process_sender, ProcessCommand::SupervisorReady
        ].unwrap();

        ok!()
    }

    fn lifecycle(&mut self) -> Result<(),Error> {
        loop {
            loop {
                let command = match try_recv!(self.supervisor_receiver) {
                    Some(command) => command,
                    None => break,
                };

                match command {
                    SupervisorCommand::ThreadCrash(thread) => return err!(Error::ThreadCrash, thread),

                    SupervisorCommand::Quit => {
                        println!("QUIT2");

                        send![
                            self.render_sender, RenderCommand::Shutdown,
                            self.controller_sender, ControllerCommand::Shutdown,
                            self.process_sender, ProcessCommand::Shutdown
                        ].unwrap();

                        return ok!();
                    },
                    _ => {},
                }
            }

            thread::sleep(Duration::new(0,20_000_000));

            try_send![self.render_sender, RenderCommand::Tick];
            try_send![self.controller_sender, ControllerCommand::Tick];
        }

        ok!()
    }

    fn synchronize_finish(&mut self) -> Result<(),Error> {
        println!("S R");
        wait![self.supervisor_receiver,
            SupervisorCommand::ThreadFinished(ThreadSource::Render) => ()
        ].unwrap();

        println!("S C");
        wait![self.supervisor_receiver,
            SupervisorCommand::ThreadFinished(ThreadSource::Controller) => ()
        ].unwrap();

        wait![self.supervisor_receiver,
            SupervisorCommand::ThreadFinished(ThreadSource::Process) => ()
        ].unwrap();

        println!("S F");
        send![
            self.render_sender, RenderCommand::SupervisorFinished,
            self.controller_sender, ControllerCommand::SupervisorFinished,
            self.process_sender, ProcessCommand::SupervisorFinished
        ];

        ok!()
    }
}
