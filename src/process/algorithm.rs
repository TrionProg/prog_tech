use nes::{ErrorInfo,ErrorInfoTrait};

use types::*;
use consts::*;

use std::thread;

use render::{RenderSender,RenderCommand};

use super::Error;
use super::TracePool;
use super::Map;

use ::Storage;

const RED:[f32;4] = [0.7,0.0,0.0,0.7];
const BLUE:[f32;4] = [0.0,0.0,0.7,0.7];
const GREEN:[f32;4] = [0.0,0.7,0.0,0.7];
const AQUA:[f32;4] = [0.0,0.7,0.7,0.7];
const YELLOW:[f32;4] = [0.7,0.7,0.0,0.7];

#[derive(Debug, Copy, Clone)]
enum Direction {
    Left,
    Right,
    Back,
    Front
}

fn calc_trace(a:(u32,u32), b:(u32,u32)) -> (Direction, f32, f32, Option<(f32,f32)>){
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

    (dir, angle, len, k)
}

pub fn add_trace(traces:&mut TracePool, storage:&Storage, a:(u32,u32), b:(u32,u32)) -> Result<TraceID,Error> {
    let (_,angle,len,_) = calc_trace(a,b);

    let trace_id=traces.insert(storage, a.0+1, a.1+1, angle, len, YELLOW)?;

    ok!(trace_id)
}

fn find_obstracle(render_sender:&mut RenderSender, map:&Map, a:(u32,u32), b:(u32,u32)) -> Result<Option<(u32,u32)>,Error> {
    let (dir,angle,len,k) = calc_trace(a,b);
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

                let is_obstracle = !map.is_floor(x, z) || !map.is_floor(x + 1, z) || !map.is_floor(x, z + 1) || !map.is_floor(x + 1, z + 1);

                if is_obstracle {
                    return ok!(Some(previous));
                }

                previous = (x, z);
                try_send!(render_sender, RenderCommand::AddTile(x,z));

                thread::sleep_ms(DELAY);
            }
        },
        _ => {},
    }

    ok!(None)
}

fn hook(render_sender:&mut RenderSender, map:&Map, mut prev:(u32,u32), obstracle_pos:(u32,u32), a:(u32,u32), b:(u32,u32), dir:Direction, len:f32) -> Result<Option<(u32,u32)>,Error> {
    let ax=a.0 as f32 + 1.0;
    let az=a.1 as f32 + 1.0;
    let bx=b.0 as f32 + 1.0;
    let bz=b.1 as f32 + 1.0;

    let mut cur=obstracle_pos;
    let mut max_dist=0.0;
    let mut max_pos=None;

    match dir {
        Direction::Front => {
            loop {
                println!("{} {}",cur.0,cur.1);
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
                        println!("New Pos {} {}",new_pos.0,new_pos.1);
                        if new_dist>max_dist {
                            max_pos=Some(new_pos);
                        }

                        /*
                        if new_dist<0.5 {
                            //TODO:совершила круг
                            break;
                        }
                        */

                        prev=cur;
                        cur=new_pos;

                        try_send!(render_sender, RenderCommand::AddTile(cur.0,cur.1));
                        thread::sleep_ms(1000);
                    }
                }
            }
        },
        _ => {}
    }

    ok!(max_pos)
}

pub fn trace_line(traces:&mut TracePool, storage:&Storage, render_sender:&mut RenderSender, map:&Map, a:(u32,u32), b:(u32,u32), trace_id:TraceID) -> Result<(),Error> {
    let obstracle=find_obstracle(render_sender, map, a,b)?;
    let (dir,_,len,_) = calc_trace(a,b);

    match obstracle {
        Some(obstracle_pos) => {
            try_send!(render_sender, RenderCommand::SetTraceColor(trace_id,RED));

            let hooks_pos=match dir {
                Direction::Front => {
                    println!("Obstracle {} {}",obstracle_pos.0,obstracle_pos.1);

                    (
                        hook(render_sender, map,obstracle_pos, (obstracle_pos.0-1,obstracle_pos.1), a, b, dir, len)?,
                        //hook(render_sender, map,obstracle_pos, (obstracle_pos.0+1,obstracle_pos.1), a, b, dir, len)?
                        None
                    )
                },
                Direction::Back => {
                    (
                        hook(render_sender, map,obstracle_pos, (obstracle_pos.0+1,obstracle_pos.1), a, b, dir, len)?,
                        hook(render_sender, map,obstracle_pos, (obstracle_pos.0-1,obstracle_pos.1), a, b, dir, len)?
                    )
                },
                _ => (None,None)
            };

            let left_traces=match hooks_pos.0 {
                Some(c) => {
                    Some((
                        (a,c,add_trace(traces, storage, a,c)?),
                        (c,b,add_trace(traces, storage, c,b)?)
                    ))
                },
                None => None
            };

            let right_traces=match hooks_pos.1 {
                Some(c) => {
                    Some((
                        (a,c,add_trace(traces, storage, a,c)?),
                        (c,b,add_trace(traces, storage, c,b)?)
                    ))
                },
                None => None
            };

            match left_traces {
                Some((t1,t2)) => {
                    trace_line(traces, storage, render_sender, map, t1.0,t1.1,t1.2)?;
                    trace_line(traces, storage, render_sender, map, t2.0,t2.1,t2.2)?;
                }
                None => {},
            }

            match right_traces {
                Some((t1,t2)) => {
                    trace_line(traces, storage, render_sender, map, t1.0,t1.1,t1.2)?;
                    trace_line(traces, storage, render_sender, map, t2.0,t2.1,t2.2)?;
                }
                None => {},
            }
        },
        None => {
            try_send!(render_sender, RenderCommand::SetTraceColor(trace_id,GREEN));
        },
    }

    //try_send!(render_sender, RenderCommand::ClearTiles);

    ok!()
}