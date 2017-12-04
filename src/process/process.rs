use std;
use nes::{ErrorInfo,ErrorInfoTrait};

use std::thread;
use std::thread::JoinHandle;

use render;
use render::RenderSender;
use render::RenderCommand;

use ::Storage;

use super::Error;
use super::ProcessCommand;

pub type ProcessSender = std::sync::mpsc::Sender<ProcessCommand>;
pub type ProcessReceiver = std::sync::mpsc::Receiver<ProcessCommand>;

pub struct Process {
    process_receiver:ProcessReceiver,
    render_sender:RenderSender,
}

impl Process {
    pub fn run(render_sender:RenderSender) -> JoinHandle<()> {
        let (process_sender, process_receiver) = std::sync::mpsc::channel();

        let join_handle=thread::Builder::new().name("Process".to_string()).spawn(move|| {
            try_send![render_sender, RenderCommand::ProcessSender(process_sender.clone())];

            let mut process=match Self::setup(process_receiver, render_sender.clone()) {
                Ok(process) => process,
                Err(error) => {
                    //error!("Process setup error: {}", error);

                    try_send![render_sender, RenderCommand::ProcessSetupError];

                    return;
                }
            };

            /*
            match process.lifecycle() {
                Ok(_) => {},
                Err(e) => {
                    use std::io::Write;
                    writeln!(&mut std::io::stderr(), "Process Error: {}!", e);
                }
            }
            */
        }).unwrap();

        join_handle
    }

    fn setup(process_receiver:ProcessReceiver, render_sender:RenderSender) -> Result<Self,Error> {
        let process=Process {
            process_receiver,
            render_sender
        };

        ok!(process)
    }
}