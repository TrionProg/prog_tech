use std;
use glutin;
use nes::{ErrorInfo,ErrorInfoTrait};
use reactor;

use types::*;

use std::thread;
use std::thread::JoinHandle;

use supervisor;
use supervisor::SupervisorSender;
use supervisor::SupervisorCommand;

use render;
use render::RenderSender;
use render::RenderCommand;

use controller;
use controller::ControllerSender;
use controller::ControllerCommand;

use ::Storage;

use super::Error;
use super::ProcessCommand;

pub type ProcessSender = reactor::Sender<ThreadSource,ProcessCommand>;
pub type ProcessReceiver = reactor::Receiver<ThreadSource,ProcessCommand>;

pub struct Process {
    process_receiver:ProcessReceiver,
    supervisor_sender:SupervisorSender,
    render_sender:RenderSender,
    controller_sender:ControllerSender,

    storage:Storage
}

impl Process{
    pub fn run()-> (JoinHandle<()>, ProcessSender) {
        let (process_sender, mut process_receiver) = reactor::create_channel(ThreadSource::Process);

        let join_handle=thread::Builder::new().name("Process".to_string()).spawn(move|| {
            let (mut supervisor_sender, mut render_sender, mut controller_sender) = Self::get_senders(&mut process_receiver).unwrap();

            println!("p1");

            let storage=Storage::new(render_sender.clone());

            println!("p2");

            let mut process=match Self::setup(
                process_receiver,
                supervisor_sender.clone(),
                render_sender.clone(),
                controller_sender.clone(),

                storage
            ) {
                Ok(process) => process,
                Err(error) => {
                    println!("Process setup error: {}", error);

                    send![
                        supervisor_sender, SupervisorCommand::ThreadCrash(ThreadSource::Process),
                        render_sender, RenderCommand::ThreadCrash(ThreadSource::Process),
                        controller_sender, ControllerCommand::ThreadCrash(ThreadSource::Process)
                    ].unwrap();

                    return;
                }
            };

            println!("p3");

            process.synchronize_setup().unwrap();

            println!("p4");

            match process.lifecycle() {
                Ok(_) => {
                    //do something

                    println!("C5");

                    process.synchronize_finish().unwrap();
                }
                Err(error) => {
                    println!("Process Error: {}!", error);

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
                                process.supervisor_sender, SupervisorCommand::ThreadCrash(ThreadSource::Process),
                                process.render_sender, RenderCommand::ThreadCrash(ThreadSource::Process),
                                process.controller_sender, ControllerCommand::ThreadCrash(ThreadSource::Process)
                            ].unwrap();
                        }
                    }
                }
            }

            println!("C6");
        }).unwrap();

        (join_handle, process_sender)
    }

    fn get_senders(receiver:&mut ProcessReceiver) -> Result<(SupervisorSender,RenderSender,ControllerSender),Error> {
        let supervisor_sender=wait![receiver,
            ProcessCommand::SupervisorSender(supervisor_sender) => supervisor_sender
        ].unwrap();

        let render_sender=wait![receiver,
            ProcessCommand::RenderSender(render_sender) => render_sender
        ].unwrap();

        let controller_sender=wait![receiver,
            ProcessCommand::ControllerSender(controller_sender) => controller_sender
        ].unwrap();

        ok!((supervisor_sender,render_sender,controller_sender))
    }

    fn setup(
        process_receiver:ProcessReceiver,
        supervisor_sender:SupervisorSender,
        render_sender:RenderSender,
        controller_sender:ControllerSender,

        storage:Storage
    ) -> Result<Self,Error> {
        let process=Process {
            process_receiver,
            supervisor_sender,
            render_sender,
            controller_sender,

            storage
        };

        ok!(process)
    }

    fn synchronize_setup(&mut self) -> Result<(),Error>{
        try_send![self.supervisor_sender, SupervisorCommand::ThreadReady(ThreadSource::Process)];

        wait![self.process_receiver,
            ProcessCommand::SupervisorReady => ()
        ].unwrap();

        ok!()
    }

    fn lifecycle(&mut self) -> Result<(),Error> {
        self.load_resources()?;
        self.create_map()?;

        loop {
            if self.handle_process_commands()? {
                println!("QUIT3");
                return ok!();
            }
        }
    }

    fn handle_process_commands(&mut self) -> Result<bool,Error> {
        loop {
            match try_recv_block!(self.process_receiver) {
                ProcessCommand::ThreadCrash(thread) => return err!(Error::ThreadCrash, thread),
                ProcessCommand::Tick => return ok!(false),
                ProcessCommand::Shutdown => return ok!(true),
                _ => unreachable!()
            }
        }
    }

    fn synchronize_finish(&mut self) -> Result<(),Error>{
        println!("C F1");
        try_send![self.supervisor_sender, SupervisorCommand::ThreadFinished(ThreadSource::Process)];
        println!("C F2");

        wait![self.process_receiver,
            ProcessCommand::SupervisorFinished => ()
        ].unwrap();

        println!("C F");

        ok!()
    }

    fn load_resources(&mut self) -> Result<(),Error>{
        use storage::{TextureStorage, MeshStorage, LodStorage};
        use storage::RgbaTexture;

        use render::storage::ObjectMesh;
        use render::storage::ObjectVertex;

        let vertex_buffer=vec![
            ObjectVertex { pos: [ -0.5, -0.5, 0.0 ], uv: [0.0, 1.0] },
            ObjectVertex { pos: [  0.5, -0.5, 0.0 ], uv: [1.0, 1.0] },
            ObjectVertex { pos: [  0.0,  0.5, 0.0 ], uv: [0.5, 0.0] }
        ];
        let lod_id=self.storage.load_lod(vertex_buffer).unwrap();

        use std::fs::{File};
        use std::io::{Read};
        use std::io::{Cursor};
        use image;

        let texture_id=RgbaTexture::load("img.png", &self.storage)?;
        let mesh=ObjectMesh::new(
            lod_id,
            texture_id
        );
        let mesh_id=self.storage.load_mesh(mesh).unwrap();

        try_send![self.render_sender, RenderCommand::ResourcesReady];

        ok!()
    }

    fn create_map(&mut self) -> Result<(),Error> {
        wait![self.process_receiver,
            ProcessCommand::ResourcesLoaded => ()
        ].unwrap();

        ok!()
    }
}