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
use super::Map;
use super::Tile;

pub type ProcessSender = reactor::Sender<ThreadSource,ProcessCommand>;
pub type ProcessReceiver = reactor::Receiver<ThreadSource,ProcessCommand>;

#[derive(Debug, Copy, Clone)]
enum Direction {
    Left,
    Right,
    Back,
    Front
}

pub struct Process {
    process_receiver:ProcessReceiver,
    supervisor_sender:SupervisorSender,
    render_sender:RenderSender,
    controller_sender:ControllerSender,

    storage:Storage,
    map:Option<Map>
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

            storage,
            map:None
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

                ProcessCommand::Algorithm(a,b) =>
                    self.algorithm(a,b)?,
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

        use render::SetSlot;

        use render::storage::ObjectMesh;
        use render::storage::ObjectVertex;

        let vertex_buffer=vec![
            ObjectVertex { pos: [ -0.5, -0.5, 0.0 ], uv: [0.0, 1.0] },
            ObjectVertex { pos: [  0.5, -0.5, 0.0 ], uv: [1.0, 1.0] },
            ObjectVertex { pos: [  0.0,  0.5, 0.0 ], uv: [0.5, 0.0] }
        ];
        let lod_id=self.storage.load_lod(vertex_buffer).unwrap();

        let texture_id=RgbaTexture::load("img.png", &self.storage)?;
        let mesh=ObjectMesh::new(
            lod_id,
            texture_id
        );
        let mesh_id=self.storage.load_mesh(mesh).unwrap();

        use std::fs::{File};
        use std::io::{Read};
        use std::io::{Cursor};
        use image;

        self.load_cursor()?;
        self.load_tile()?;
        self.load_textures()?;
        self.load_floor()?;
        self.load_walls()?;
        self.load_holes()?;


        try_send![self.render_sender, RenderCommand::ResourcesReady];

        ok!()
    }

    fn load_cursor(&mut self) -> Result<(),Error> {
        use render::SetSlot;
        use storage::{MeshStorage, LodStorage};
        use storage::RgbaTexture;
        use storage::TextureStorage;

        use render::storage::ObjectMesh;
        use render::storage::ObjectVertex;

        let mut buffer=Vec::with_capacity(1*6);

        let top=[
            ObjectVertex{ pos:[0.0, 0.0, 0.0], uv:[0.0, 0.0]},
            ObjectVertex{ pos:[2.0, 0.0, 0.0], uv:[1.0, 0.0]},
            ObjectVertex{ pos:[2.0, 0.0, 2.0], uv:[1.0, 1.0]},
            ObjectVertex{ pos:[2.0, 0.0, 2.0], uv:[1.0, 1.0]},
            ObjectVertex{ pos:[0.0, 0.0, 2.0], uv:[0.0, 1.0]},
            ObjectVertex{ pos:[0.0, 0.0, 0.0], uv:[0.0, 0.0]},
        ];

        buffer.extend_from_slice(&top);
        let lod_id=self.storage.load_lod(buffer).unwrap();

        //Cursor
        let texture_id=RgbaTexture::load("textures/cursor.png", &self.storage)?;

        let mesh=ObjectMesh::new(
            lod_id,texture_id
        );

        let mesh_id=self.storage.load_mesh(mesh)?;

        try_send![self.render_sender, SetSlot::Cursor(mesh_id).into()];

        //CursorA
        let texture_id=RgbaTexture::load("textures/cursor_a.png", &self.storage)?;

        let mesh=ObjectMesh::new(
            lod_id,texture_id
        );

        let mesh_id=self.storage.load_mesh(mesh)?;

        try_send![self.render_sender, SetSlot::CursorA(mesh_id).into()];

        //CursorB
        let texture_id=RgbaTexture::load("textures/cursor_b.png", &self.storage)?;

        let mesh=ObjectMesh::new(
            lod_id,texture_id
        );

        let mesh_id=self.storage.load_mesh(mesh)?;

        try_send![self.render_sender, SetSlot::CursorB(mesh_id).into()];
        ok!()
    }

    fn load_tile(&mut self) -> Result<(),Error> {
        use render::SetSlot;
        use storage::{MeshStorage, LodStorage};
        use storage::RgbaTexture;
        use storage::TextureStorage;

        use render::storage::ObjectMesh;
        use render::storage::ObjectVertex;

        let mut buffer=Vec::with_capacity(1*6);

        let top=[
            ObjectVertex{ pos:[0.2, 0.0, 0.2], uv:[0.0, 0.0]},
            ObjectVertex{ pos:[0.8, 0.0, 0.2], uv:[1.0, 0.0]},
            ObjectVertex{ pos:[0.8, 0.0, 0.8], uv:[1.0, 1.0]},
            ObjectVertex{ pos:[0.8, 0.0, 0.8], uv:[1.0, 1.0]},
            ObjectVertex{ pos:[0.2, 0.0, 0.8], uv:[0.0, 1.0]},
            ObjectVertex{ pos:[0.2, 0.0, 0.2], uv:[0.0, 0.0]},
        ];

        buffer.extend_from_slice(&top);

        let texture_id=RgbaTexture::load("textures/tile.png", &self.storage)?;
        let lod_id=self.storage.load_lod(buffer).unwrap();

        let mesh=ObjectMesh::new(
            lod_id,texture_id
        );

        let mesh_id=self.storage.load_mesh(mesh)?;

        try_send![self.render_sender, SetSlot::Tile(mesh_id).into()];

        ok!()
    }

    fn load_textures(&mut self) -> Result<(),Error> {
        use render::SetSlot;
        use storage::RgbaTexture;
        use storage::TextureStorage;

        for i in 0..5 {
            let file_name=format!("textures/terrain{}.png",i);
            let texture_id=RgbaTexture::load(file_name.as_str(), &self.storage)?;

            try_send![self.render_sender, SetSlot::TerrainTexture(i,texture_id).into()];
        }

        ok!()
    }

    fn load_floor(&mut self) -> Result<(),Error> {
        use render::SetSlot;
        use storage::{MeshStorage, LodStorage};

        use render::storage::TerrainMesh;
        use render::storage::ObjectVertex;

        let mut buffer=Vec::with_capacity(1*6);

        let top=[
            ObjectVertex::new([0, 0,  0], [0, 0]),
            ObjectVertex::new([ 1, 0,  0], [1, 0]),
            ObjectVertex::new([ 1,  1,  0], [1, 1]),
            ObjectVertex::new([ 1,  1,  0], [1, 1]),
            ObjectVertex::new([0,  1,  0], [0, 1]),
            ObjectVertex::new([0, 0,  0], [0, 0]),
        ];

        buffer.extend_from_slice(&top);

        let lod_id=self.storage.load_lod(buffer).unwrap();
        let mesh=TerrainMesh::new(
            lod_id
        );

        let mesh_id=self.storage.load_mesh(mesh)?;

        try_send![self.render_sender, SetSlot::FloorMesh(mesh_id).into()];

        ok!()
    }

    fn load_walls(&mut self) -> Result<(),Error> {
        use render::SetSlot;
        use storage::{MeshStorage, LodStorage};

        use render::storage::TerrainMesh;
        use render::storage::ObjectVertex;

        for i in 0..16 {
            let mut buffer=Vec::with_capacity(6*6);

            let top=[
                ObjectVertex::new([0, 0,  2], [0, 0]),
                ObjectVertex::new([ 1, 0,  2], [1, 0]),
                ObjectVertex::new([ 1,  1,  2], [1, 1]),
                ObjectVertex::new([ 1,  1,  2], [1, 1]),
                ObjectVertex::new([0,  1,  2], [0, 1]),
                ObjectVertex::new([0, 0,  2], [0, 0]),
            ];

            let right=[
                ObjectVertex::new([1, 0,  2], [1, 0]),
                ObjectVertex::new([1,  1,  2], [0, 0]),
                ObjectVertex::new([1,  1, 0], [0, 2]),
                ObjectVertex::new([1,  1, 0], [0, 2]),
                ObjectVertex::new([1, 0, 0], [1, 2]),
                ObjectVertex::new([1, 0,  2], [1, 0]),
            ];

            let left=[
                ObjectVertex::new([0, 1, 2], [1, 0]),
                ObjectVertex::new([0, 0, 0], [0, 2]),
                ObjectVertex::new([0, 0, 2], [0, 0]),
                ObjectVertex::new([0, 1, 2], [1, 0]),
                ObjectVertex::new([0, 1, 0], [1, 2]),
                ObjectVertex::new([0, 0, 0], [0, 2]),
            ];

            let front=[
                ObjectVertex::new([0,  1,  2], [0, 0]),
                ObjectVertex::new([0,  1, 0], [0, 2]),
                ObjectVertex::new([ 1,  1, 0], [1, 2]),
                ObjectVertex::new([ 1,  1, 0], [1, 2]),
                ObjectVertex::new([ 1,  1,  2], [1, 0]),
                ObjectVertex::new([0,  1,  2], [0, 0]),
            ];

            let back=[
                ObjectVertex::new([ 1, 0,  2], [0, 0]),
                ObjectVertex::new([0, 0,  2], [1, 0]),
                ObjectVertex::new([0, 0, 0], [1, 2]),
                ObjectVertex::new([0, 0, 0], [1, 2]),
                ObjectVertex::new([ 1, 0, 0], [0, 2]),
                ObjectVertex::new([ 1, 0,  2], [0, 0]),
            ];

            buffer.extend_from_slice(&top);

            if (i & 1) >0 {
                buffer.extend_from_slice(&right);
            }

            if (i & 1<<1) >0 {
                buffer.extend_from_slice(&left);
            }

            if (i & 1<<2) >0 {
                buffer.extend_from_slice(&front);
            }

            if (i & 1<<3) >0 {
                buffer.extend_from_slice(&back);
            }

            let lod_id=self.storage.load_lod(buffer).unwrap();
            let mesh=TerrainMesh::new(
                lod_id
            );

            let mesh_id=self.storage.load_mesh(mesh)?;

            try_send![self.render_sender, SetSlot::WallMesh(i,mesh_id).into()];
        }

        ok!()
    }

    fn load_holes(&mut self) -> Result<(),Error> {
        use render::SetSlot;
        use storage::{MeshStorage, LodStorage};

        use render::storage::TerrainMesh;
        use render::storage::ObjectVertex;

        for i in 0..16 {
            let mut buffer=Vec::with_capacity(6*6);

            let top=[
                ObjectVertex::new([0, 0,  -2], [0, 0]),
                ObjectVertex::new([ 1, 0,  -2], [1, 0]),
                ObjectVertex::new([ 1,  1,  -2], [1, 1]),
                ObjectVertex::new([ 1,  1,  -2], [1, 1]),
                ObjectVertex::new([0,  1,  -2], [0, 1]),
                ObjectVertex::new([0, 0,  -2], [0, 0]),
            ];

            let right=[
                ObjectVertex::new([ 1,  1,  2-2], [1, 0]),
                ObjectVertex::new([ 1,  1, -2], [1, 2]),
                ObjectVertex::new([ 1, 0, -2], [0, 2]),

                ObjectVertex::new([ 1, 0, 0-2], [0, 2]),
                ObjectVertex::new([ 1, 0,  2-2], [0, 0]),
                ObjectVertex::new([ 1,  1,  2-2], [1, 0]),
            ];

            let left=[
                ObjectVertex::new([0, 0,  2-2], [1, 0]),
                ObjectVertex::new([0,  1,  2-2], [0, 0]),
                ObjectVertex::new([0,  1, 0-2], [0, 2]),
                ObjectVertex::new([0,  1, 0-2], [0, 2]),
                ObjectVertex::new([0, 0, 0-2], [1, 2]),
                ObjectVertex::new([0, 0,  2-2], [1, 0]),
            ];

            let front=[
                ObjectVertex::new([0,  1,  2-2], [1, 0]),
                ObjectVertex::new([0,  1, 0-2], [1, 2]),
                ObjectVertex::new([ 1,  1, 0-2], [0, 2]),
                ObjectVertex::new([ 1,  1, 0-2], [0, 2]),
                ObjectVertex::new([ 1,  1,  2-2], [0, 0]),
                ObjectVertex::new([0,  1,  2-2], [1, 0]),
            ];

            let back=[
                ObjectVertex::new([ 1, 0,  2-2], [1, 0]),
                ObjectVertex::new([0, 0,  2-2], [0, 0]),
                ObjectVertex::new([0, 0, 0-2], [0, 2]),
                ObjectVertex::new([0, 0, 0-2], [0, 2]),
                ObjectVertex::new([ 1, 0, 0-2], [1, 2]),
                ObjectVertex::new([ 1, 0,  2-2], [1, 0]),
            ];

            buffer.extend_from_slice(&top);

            if (i & 1) >0 {
                buffer.extend_from_slice(&right);
            }

            if (i & 1<<1) >0 {
                buffer.extend_from_slice(&left);
            }

            if (i & 1<<2) >0 {
                buffer.extend_from_slice(&front);
            }

            if (i & 1<<3) >0 {
                buffer.extend_from_slice(&back);
            }

            let lod_id=self.storage.load_lod(buffer).unwrap();
            let mesh=TerrainMesh::new(
                lod_id
            );

            let mesh_id=self.storage.load_mesh(mesh)?;

            try_send![self.render_sender, SetSlot::HoleMesh(i,mesh_id).into()];
        }

        ok!()
    }

    fn create_map(&mut self) -> Result<(),Error> {
        use std::io::{BufReader,BufRead};
        use std::fs::File;
        use consts::MAP_SIZE;

        wait![self.process_receiver,
            ProcessCommand::ResourcesLoaded => ()
        ].unwrap();

        let f = File::open("map.txt").unwrap();
        let mut reader = BufReader::new(f);

        let mut map=Map::new();

        for (z,line_res) in reader.lines().enumerate() {
            let line=match line_res{
                Ok(line) => line,
                Err(_) => break
            };

            let chars:Vec<char>=line.chars().collect();

            for x in 0..MAP_SIZE {
                let index=match chars[x*2+1] {
                    '0' => 0,
                    '1' => 1,
                    '2' => 2,
                    '3' => 3,
                    '4' => 4,
                    _ => 0
                };

                if index>4 {
                    panic!("{} {}",x,z);
                }

                let tile=match chars[x*2] {
                    'w' => Tile::Wall(index),
                    'f' => Tile::Floor(index),
                    'h' => Tile::Hole(index),
                    _ => Tile::Air,
                };

                map.tiles[x][z]=tile;
            }
        }

        try_send![self.render_sender, RenderCommand::CreateMap];

        for z in 0..MAP_SIZE {
            for x in 0..MAP_SIZE {
                thread::sleep_ms(10);
                try_send![self.render_sender, RenderCommand::LoadTile(x,z,map.tiles[x][z]) ];
            }
        }

        self.map=Some(map);

        ok!()
    }

    fn algorithm(&mut self, a:(u32,u32), b:(u32,u32)) -> Result<(),Error> {
        self.algorithm_trace_line(a,b)?;

        ok!()
    }

    fn algorithm_trace_line(&mut self, a:(u32,u32), b:(u32,u32)) -> Result<(),Error> {
        use std::f32::consts::PI;

        let ax=a.0 as f32 + 1.0;
        let az=a.1 as f32 + 1.0;
        let bx=b.0 as f32 + 1.0;
        let bz=b.1 as f32 + 1.0;
        let len=((bx-ax).powi(2) + (bz-az).powi(2)).sqrt();

        let sin=if a.0==b.0 {
            None
        }else{
            Some((bx-ax)/len)
        };

        let (dir,angle)=match sin {
            None => {
                if az<bz {
                    (Direction::Front,0.0)
                }else{
                    (Direction::Back,PI)
                }
            },
            Some(sin) => {
                let asin=sin.asin();
                let angle=if az <= bz {
                    asin
                }else if asin>0.0 {
                    PI-asin
                }else{
                    -PI-asin
                };

                if angle >= -PI/4.0 && angle <= PI/4.0 {
                    (Direction::Front,angle)
                }else if angle >= PI-PI/4.0 || angle <= -PI+PI/4.0 {
                    (Direction::Back,angle)
                }else if angle > PI/4.0 && angle < PI-PI/4.0 {
                    (Direction::Right,angle)
                }else{
                    (Direction::Left,angle)
                }
            }
        };

        println!("Direction {:?}", dir);

        try_send!(self.controller_sender, ControllerCommand::AlgorithmEnd);

        ok!()
    }
}