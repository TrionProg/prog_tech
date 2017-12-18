use std;
use nes::{ErrorInfo,ErrorInfoTrait};
use reactor;

use types::*;

use std::thread;
use std::thread::JoinHandle;

use render;
use render::RenderSender;
use render::RenderCommand;
use render::StorageSender;

use ::Storage;

use super::Error;
use super::ProcessCommand;

pub type ProcessSender = reactor::Sender<ThreadSource,ProcessCommand>;
pub type ProcessReceiver = reactor::Receiver<ThreadSource,ProcessCommand>;

pub struct Process {
    process_receiver:ProcessReceiver,
    render_sender:RenderSender,
    storage:Storage
}

impl Process {
    pub fn run(mut render_sender:RenderSender, storage_sender:StorageSender) -> JoinHandle<()> {
        let (process_sender, process_receiver):(ProcessSender,ProcessReceiver) = reactor::create_channel(ThreadSource::Process);

        let join_handle=thread::Builder::new().name("Process".to_string()).spawn(move|| {
            send![
                render_sender, RenderCommand::ProcessSender(process_sender.clone())
            ].unwrap();

            let storage=Storage::new(storage_sender);

            let mut process=match Self::setup(process_receiver, render_sender.clone(), storage) {
                Ok(process) => process,
                Err(error) => {
                    println!("Process setup error: {}", error);

                    send![
                        render_sender, RenderCommand::ProcessSetupError
                    ].unwrap();

                    return;
                }
            };

            process.synchronize_setup().unwrap();

            match process.lifecycle() {
                Ok(_) => {
                    //do something

                    process.synchronize_finish().unwrap();
                }
                Err(error) => {
                    println!("Process Error: {}", error);

                    match error {
                        Error::RenderThreadCrash(_,source) => {
                            /*
                            if source==ThreadSource::Process {
                                try_send![storage.disk_sender, DiskCommand::IpcListenerThreadCrash(source)];
                            }
                            */
                        }
                        _ => {
                            send![
                                process.render_sender, RenderCommand::ProcessThreadCrash(ThreadSource::Process)
                            ].unwrap();
                        }
                    }
                }
            }
        }).unwrap();

        join_handle
    }

    fn setup(process_receiver:ProcessReceiver, render_sender:RenderSender, storage:Storage) -> Result<Self,Error> {
        let process=Process {
            process_receiver,
            render_sender,
            storage
        };

        ok!(process)
    }

    fn synchronize_setup(&mut self) -> Result<(),Error>{
        wait![self.process_receiver,
            ProcessCommand::RenderIsReady => ()
        ].unwrap();

        try_send![self.render_sender, RenderCommand::ProcessIsReady];

        ok!()
        /*
        let mut ipc_listener_is_ready=false;
        let mut disk_is_ready=false;

        for _ in 0..2 {
            match self.storage_receiver.recv() {
                Ok( StorageCommand::IpcListenerIsReady ) => ipc_listener_is_ready=true,
                Ok( StorageCommand::DiskIsReady ) => disk_is_ready=true,
                _ => recv_error!(StorageCommand::IpcListenerOrDiskIsReady),
            }
        }

        if ipc_listener_is_ready && disk_is_ready {
            try_send![self.ipc_listener_sender, IpcListenerCommand::StorageIsReady];
            try_send![self.disk_sender, DiskCommand::StorageIsReady];
        }else if !ipc_listener_is_ready{
            recv_error!(StorageCommand::IpcListenerIsReady);
        }else if !disk_is_ready{
            recv_error!(StorageCommand::DiskIsReady);
        }
        */
        /*
        let mut render_is_ready=false;

        for _ in 0..1 {
            match self.process_receiver.recv() {
                Ok( ProcessCommand::RenderIsReady ) => render_is_ready=true,
                _ => recv_error!(ProcessCommand::RenderOrDiskIsReady),
            }
        }

        if render_is_ready {
            try_send![self.render_sender, RenderCommand::ProcessIsReady];
        }
        */
    }

    fn lifecycle(&mut self) -> Result<(),Error> {
        self.lifecycle_process()?;

        ok!()
    }

    fn lifecycle_process(&mut self) -> Result<(),Error> {
        use storage::{TextureStorage, MeshStorage, LodStorage};

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

        let mut buf = Vec::new();
        let fullpath = "img.png";//&Path::new("img.png");//.join(&path);
        let mut file = match File::open(&fullpath) {
            Ok(file) => file,
            Err(err) => {
                panic!("Can`t open file '{}' ({})", fullpath, err);
            },
        };
        let cursor=match file.read_to_end(&mut buf) {
            Ok(_) => Cursor::new(buf),
            Err(err) => {
                panic!("Can`t read file '{}' ({})", fullpath, err);
            },
        };

        let image_buffer = image::load(cursor, image::PNG).unwrap().to_rgba();

        let texture_id=self.storage.load_texture(image_buffer).unwrap();
        let mesh=ObjectMesh::new(
            lod_id,
            texture_id
        );
        let mesh_id=self.storage.load_mesh(mesh).unwrap();

        loop {
            loop {
                let command = match try_recv!(self.process_receiver) {
                    Some(command) => command,
                    None => break,
                };

                match command {
                    ProcessCommand::RenderThreadCrash(source) => return err!(Error::RenderThreadCrash, source),

                    ProcessCommand::Quit => {
                        try_send![self.render_sender, RenderCommand::Shutdown];
                        return ok!();
                    },
                    _ => {},
                }
            }

            thread::sleep(Duration::new(1,0));
        }
    }

    fn synchronize_finish(&mut self) -> Result<(),Error>{
        wait![self.process_receiver,
            ProcessCommand::RenderFinished => ()
        ].unwrap();

        try_send![self.render_sender, RenderCommand::ProcessFinished];

        ok!()
        /*
        let mut ipc_listener_finished=false;
        let mut disk_finished=false;

        for _ in 0..2 {
            match self.storage_receiver.recv() {
                Ok( StorageCommand::IpcListenerFinished ) => ipc_listener_finished=true,
                Ok( StorageCommand::DiskFinished ) => disk_finished=true,
                _ => recv_error!(StorageCommand::IpcListenerOrDiskIsReady),
            }
        }

        if ipc_listener_finished && disk_finished {
            try_send![self.ipc_listener_sender, IpcListenerCommand::StorageFinished];
            try_send![self.disk_sender, DiskCommand::StorageFinished];
        }else if !ipc_listener_finished{
            recv_error!(StorageCommand::IpcListenerIsReady);
        }else if !disk_finished{
            recv_error!(StorageCommand::DiskIsReady);
        }
        */
        /*
        let mut render_finished=false;

        for _ in 0..1 {
            match self.process_receiver.recv() {
                Ok( ProcessCommand::RenderFinished ) => render_finished=true,
                _ => recv_error!(ProcessCommand::RenderOrDiskIsReady),
            }
        }

        if render_finished {
            try_send![self.render_sender, RenderCommand::ProcessFinished];
        }
        */
    }
}