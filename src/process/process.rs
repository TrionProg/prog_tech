use std;
use glutin;
use nes::{ErrorInfo,ErrorInfoTrait};
use reactor;

use types::*;
use consts::*;

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
use super::TracePool;
use super::algorithm::*;

pub type ProcessSender = reactor::Sender<ThreadSource,ProcessCommand>;
pub type ProcessReceiver = reactor::Receiver<ThreadSource,ProcessCommand>;

pub struct Process {
    process_receiver:ProcessReceiver,
    supervisor_sender:SupervisorSender,
    render_sender:RenderSender,
    controller_sender:ControllerSender,

    storage:Storage,
    map:Option<Map>,
    traces:TracePool,
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
        let traces=TracePool::new(render_sender.clone());

        let process=Process {
            process_receiver,
            supervisor_sender,
            render_sender,
            controller_sender,

            storage,
            map:None,
            traces
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
        let map=match self.map {
            Some(ref mut map) => map,
            None => panic!("No map")
        };


        let trace_id=add_trace(&mut self.traces, &self.storage, Pos2D::new(a.0,a.1), Pos2D::new(b.0,b.1))?;

        trace_line(&mut self.traces, &self.storage, &mut self.render_sender, map,
                   Pos2D::new(a.0,a.1), Pos2D::new(b.0,b.1), trace_id, HookMode::Unreachable)?;
        try_send!(self.controller_sender, ControllerCommand::AlgorithmEnd);

        ok!()
    }
/*
    fn add_trace(&mut self, a:(u32,u32), b:(u32,u32)) -> Result<TraceID,Error> {
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

        let angle=match sin {
            None => {
                if az<bz {
                    0.0
                }else{
                    PI
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

                angle
            }
        };

        let trace_id=self.traces.insert(&self.storage, a.0+1, a.1+1, angle, len, YELLOW)?;

        ok!(trace_id)
    }

    fn algorithm_trace_line(&mut self, a:(u32,u32), b:(u32,u32), trace_id:TraceID) -> Result<(),Error> {
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

        let map=match self.map {
            Some(ref map) => map,
            None => panic!("No map")
        };

        let k=match dir {
            Direction::Front | Direction::Back => {
                if a.0==b.0 {
                    None
                }else{
                    let k=(bz-az)/(bx-ax);
                    let b=az-k*ax;
                    Some((k,b))
                }
            },
            _ => {
                if a.1==b.1 {
                    None
                }else{
                    let k=(bx-ax)/(bz-az);
                    let b=ax-k*az;
                    Some((k,b))
                }
            }
        };

        let obstracle= {
            let render_sender = &mut self.render_sender;

            fn find_obstracle () {
                let mut previous = (a.0, a.1);

                match dir {
                    Direction::Front => {
                        for z in a.1..(b.1 + 1) {
                            let x = match k {
                                None => a.0 as u32,
                                Some((k, b)) => {
                                    let x = (z as f32 - b) / k;
                                    x as u32
                                }
                            };

                            let no_obstracle = map.is_floor(x, z + 1) && map.is_floor(x + 1, z + 1);

                            if no_obstracle {
                                previous = (x, z);
                                try_send!(render_sender, RenderCommand::AddTile(x,z));
                            } else {
                                return ok!(Some(previous));
                            }

                            thread::sleep_ms(DELAY);
                        }
                    },
                    _ => {},
                }

                ok!(None)
            };

            find_obstracle()?
        };

        match obstracle {
            Some(obstracle_pos) => {
                let mut hook=|mut prev:(u32,u32),render_sender:&mut RenderSender|{
                    let mut cur=obstracle_pos;
                    let mut max_dist=0.0;
                    let mut max_pos=None;

                    match dir {
                        Direction::Front => {
                            loop {
                                let mut variants=[None,None,None,None];

                                //front
                                let pos=(cur.0,cur.1+1);
                                variants[0]=if pos!=prev && pos.1 < (MAP_SIZE as u32 -1) && map.is_floor(pos.0, pos.1+1) && map.is_floor(pos.0 + 1, pos.1 + 1) {
                                    let dist=((bz-az)*pos.0 as f32 - (bx-ax)*(pos.1) as f32 + bx*az - bz*ax).abs() / len;
                                    Some((pos,dist))
                                }else{
                                    None
                                };

                                //right
                                let pos=(cur.0+1,cur.1);
                                variants[1]=if pos!=prev && pos.0 < (MAP_SIZE as u32 -1) && map.is_floor(pos.0+1, pos.1) && map.is_floor(pos.0 + 1, pos.1 + 1) {
                                    let dist = ((bz - az) * pos.0 as f32 - (bx - ax) * pos.1 as f32 + bx * az - bz * ax).abs() / len;
                                    Some((pos,dist))
                                }else{
                                    None
                                };

                                //left
                                variants[2]=if cur.0>0 {
                                    let pos=(cur.0-1,cur.1);

                                    if pos!=prev && map.is_floor(pos.0, pos.1) && map.is_floor(pos.0, pos.1 + 1) {
                                        let dist = ((bz - az) * pos.0 as f32 - (bx - ax) * pos.1 as f32 + bx * az - bz * ax).abs() / len;
                                        Some((pos,dist))
                                    }else{
                                        None
                                    }
                                }else{
                                    None
                                };

                                //back
                                variants[3]=if cur.1>0 {
                                    let pos=(cur.0,cur.1-1);

                                    if pos!=prev && map.is_floor(pos.0, pos.1) && map.is_floor(pos.0 + 1, pos.1) {
                                        let dist = ((bz - az) * pos.0 as f32 - (bx - ax) * pos.1 as f32 + bx * az - bz * ax).abs() / len;
                                        Some((pos,dist))
                                    }else{
                                        None
                                    }
                                }else{
                                    None
                                };

                                let (new_pos,new_dist) = {
                                    let mut new_pos=None;
                                    let mut min_dist=9000.0;

                                    for variant in variants.iter() {
                                        match *variant {
                                            Some((pos,dist)) => {
                                                if dist < min_dist {
                                                    new_pos=Some(pos);
                                                    min_dist=dist;
                                                }
                                            },
                                            None => {},
                                        }
                                    }

                                    (new_pos,min_dist)
                                };

                                match new_pos {
                                    None => {
                                        let tmp=cur;
                                        cur=prev;
                                        prev=cur;
                                    },
                                    Some(new_pos) => {
                                        if new_dist>max_dist {
                                            max_pos=Some(new_pos);
                                        }

                                        if new_dist<1.0 {
                                            //TODO:совершила круг
                                            break;
                                        }

                                        prev=cur;
                                        cur=new_pos;

                                        try_send!(render_sender, RenderCommand::AddTile(cur.0,cur.1));
                                        thread::sleep_ms(DELAY);
                                    }
                                }
                            }
                        },
                        _ => {}
                    }

                    ok!(max_pos)
                };

                try_send!(self.render_sender, RenderCommand::SetTraceColor(trace_id,RED));

                let hooks_pos=match dir {
                    Direction::Front => {
                        (hook((obstracle_pos.0+1,obstracle_pos.1),&mut self.render_sender)?,hook((obstracle_pos.0-1,obstracle_pos.1),&mut self.render_sender)?)
                    },
                    Direction::Back => {
                        (hook((obstracle_pos.0-1,obstracle_pos.1),&mut self.render_sender)?,hook((obstracle_pos.0+1,obstracle_pos.1),&mut self.render_sender)?)
                    },
                    _ => (None,None)
                };

                let left_traces=match hooks_pos.0 {
                    Some(p) => {
                        let a=a;
                        let b=p;
                        let c=b;

                        Some(((a,b,self.add_trace(a,b)?),(b,c,self.add_trace(b,c)?)))
                    },
                    None => None
                };

                let right_traces=match hooks_pos.1 {
                    Some(p) => {
                        let a=a;
                        let b=p;
                        let c=b;

                        Some(((a,b,self.add_trace(a,b)?),(b,c,self.add_trace(b,c)?)))
                    },
                    None => None
                };

                match left_traces {
                    Some((t1,t2)) => {
                        self.algorithm_trace_line(t1.0,t1.1,t1.2)?;
                        self.algorithm_trace_line(t2.0,t2.1,t2.2)?;
                    }
                    None => {},
                }

                match right_traces {
                    Some((t1,t2)) => {
                        self.algorithm_trace_line(t1.0,t1.1,t1.2)?;
                        self.algorithm_trace_line(t2.0,t2.1,t2.2)?;
                    }
                    None => {},
                }
            },
            None => {
                try_send!(self.render_sender, RenderCommand::SetTraceColor(trace_id,GREEN));
            },
        }

        try_send!(self.render_sender, RenderCommand::ClearTiles);

        ok!()
    }
    */
}