use render;
use nes::{ErrorInfo,ErrorInfoTrait};

use types::*;

use object_pool::growable::Pool;
use render::{RenderSender,RenderCommand};
use render::storage::{TraceMesh,TraceLod};
use render::storage::TraceVertex;

use super::Error;

use ::Storage;

use storage::storage::{MeshStorage,LodStorage};

pub struct TracePool {
    pool:Pool<(), ()>,
    render_sender:RenderSender,
}

impl TracePool {
    pub fn new(render_sender:RenderSender) -> Self {
        TracePool {
            pool:Pool::new(),
            render_sender
        }
    }

    pub fn insert(&mut self, storage:&Storage,
                  x:u32,
                  z:u32,
                  angle:f32,
                  len:f32,
                  color:[f32;4]
    ) -> Result<TraceID,Error> {
        let top=[
            TraceVertex::new(-0.05, 0.15, len),
            TraceVertex::new(-0.05, 0.15, 0.0),
            TraceVertex::new(0.05, 0.15, 0.0),
            TraceVertex::new(-0.05, 0.15, len),
            TraceVertex::new(0.05, 0.15, 0.0),
            TraceVertex::new(0.05, 0.15, len),
        ];

        let left=[
            TraceVertex::new(-0.05, 0.15, len),
            TraceVertex::new(-0.05, 0.0, len),
            TraceVertex::new(-0.05, 0.0, 0.0),
            TraceVertex::new(-0.05, 0.15, len),
            TraceVertex::new(-0.05, 0.0, 0.0),
            TraceVertex::new(-0.05, 0.15, 0.0),
        ];

        let front=[
            TraceVertex::new(-0.05, 0.15, 0.0),
            TraceVertex::new(-0.05, 0.0, 0.0),
            TraceVertex::new(0.05, 0.0, 0.0),
            TraceVertex::new(-0.05, 0.15, 0.0),
            TraceVertex::new(0.05, 0.0, 0.0),
            TraceVertex::new(0.05, 0.15, 0.0),
        ];

        let right=[
            TraceVertex::new(0.05, 0.15, len),
            TraceVertex::new(0.05, 0.0, len),
            TraceVertex::new(0.05, 0.0, 0.0),
            TraceVertex::new(0.05, 0.15, len),
            TraceVertex::new(0.05, 0.0, 0.0),
            TraceVertex::new(0.05, 0.15, 0.0),
        ];

        let back=[
            TraceVertex::new(-0.05, 0.15, len),
            TraceVertex::new(-0.05, 0.0, len),
            TraceVertex::new(0.05, 0.0, len),
            TraceVertex::new(-0.05, 0.15, len),
            TraceVertex::new(0.05, 0.0, len),
            TraceVertex::new(0.05, 0.15, len),
        ];

        let mut buffer=Vec::with_capacity(6*6);
        buffer.extend_from_slice(&top);
        buffer.extend_from_slice(&left);
        buffer.extend_from_slice(&right);
        buffer.extend_from_slice(&front);
        buffer.extend_from_slice(&back);

        let lod_id=storage.load_lod(buffer).unwrap();
        let mesh=TraceMesh::new(
            lod_id
        );

        let mesh_id=storage.load_mesh(mesh)?;

        let trace=render::Trace::new(
            x,
            z,
            angle,
            color,
            mesh_id
        );

        let id=self.pool.insert(());
        let id=TraceID::new(id);

        try_send!(self.render_sender, RenderCommand::CreateTrace(trace));

        ok!(id)
    }

    pub fn delete(&mut self,id:TraceID) -> Result<(),Error>{
        self.pool.remove(id.get_id());

        try_send!(self.render_sender, RenderCommand::DeleteTrace(id));

        ok!()
    }
}