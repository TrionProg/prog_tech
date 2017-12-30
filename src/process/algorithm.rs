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

const MAP_SIZE1:u32 = MAP_SIZE as u32 - 1;
const MAP_SIZE2:u32 = MAP_SIZE as u32 - 2;

#[derive(Debug, Copy, Clone)]
enum Direction {
    Left,
    Right,
    Back,
    Front
}

#[derive(Copy,Clone,Eq,PartialEq)]
pub struct Pos2D {
    x:u32,
    z:u32,
}

impl Pos2D {
    pub fn new(x:u32,z:u32) -> Self {
        Pos2D {
            x,
            z
        }
    }
}

fn calc_trace(a:Pos2D, b:Pos2D) -> (Direction, f32, f32, Option<(f32,f32)>){
    use std::f32::consts::PI;

    let ax=a.x as f32 + 1.0;
    let az=a.z as f32 + 1.0;
    let bx=b.x as f32 + 1.0;
    let bz=b.z as f32 + 1.0;
    let len=((bx-ax).powi(2) + (bz-az).powi(2)).sqrt();

    let sin=if a.x==b.x {
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
            if a.x==b.x {
                None
            }else{
                let k=(bz-az)/(bx-ax);
                let b=az-k*ax;
                Some((k,b))
            }
        },
        _ => {
            if a.z==b.z {
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

pub fn add_trace(traces:&mut TracePool, storage:&Storage, a:Pos2D, b:Pos2D) -> Result<TraceID,Error> {
    let (_,angle,len,_) = calc_trace(a,b);

    let trace_id=traces.insert(storage, a.x+1, a.z+1, angle, len, YELLOW)?;

    ok!(trace_id)
}

fn find_obstracle(render_sender:&mut RenderSender, map:&mut Map, a:Pos2D, b:Pos2D) -> Result<Option<Pos2D>,Error> {
    let (dir,angle,len,k) = calc_trace(a,b);
    let mut previous = a;

    match dir {
        Direction::Front => {
            for z in a.z..(b.z + 1) {
                let x = match k {
                    None => a.x as u32,
                    Some((k, b)) => {
                        let x = (z as f32 - b) / k;
                        x as u32
                    }
                };

                let is_obstracle = !map.is_floor(x, z) || !map.is_floor(x + 1, z) || !map.is_floor(x, z + 1) || !map.is_floor(x + 1, z + 1);

                if is_obstracle {
                    return ok!(Some(previous));
                }

                previous = Pos2D::new(x, z);
                try_send!(render_sender, RenderCommand::AddTile(x,z,true));

                thread::sleep_ms(DELAY);
            }
        },
        _ => {},
    }

    ok!(None)
}

fn hook(render_sender:&mut RenderSender, map:&Map, obstracle_pos:Pos2D, obstracle_dir:Direction, clockwise:bool, a:Pos2D, b:Pos2D, len:f32) -> Result<Option<Pos2D>,Error> {
    let ax=a.x as f32 + 1.0;
    let az=a.z as f32 + 1.0;
    let bx=b.x as f32 + 1.0;
    let bz=b.z as f32 + 1.0;

    let mut c=obstracle_pos;
    let mut move_dir=None;

    let mut max_dist=0.0;
    //let mut max_pos=None;

    let (mut p,mut move_dir)=match obstracle_dir {
        Direction::Front => {
            let is_left=!map.is_floor(c.x,c.z+2);
            let is_right=!map.is_floor(c.x+1,c.z+2);

            if clockwise {
                let p=if is_right {
                    Pos2D::new(c.x+1,c.z+2)
                }else{
                    Pos2D::new(c.x,c.z+2)
                };
                
                (p,Some(Direction::Right))//TODO veillecht None?
            }else{
                let p=if is_left {
                    Pos2D::new(c.x,c.z+2)
                }else{
                    Pos2D::new(c.x+1,c.z+2)
                };

                (p,Some(Direction::Left))
            }
        },
        _ => unimplemented!(),
    };

    for i in 0..60 {
        println!("CUR:{} {} POINT:{} {}",c.x,c.z,p.x,p.z);
        if i<35 {
            thread::sleep_ms(100);
        }else {
            thread::sleep_ms(1000);
        }
        try_send!(render_sender, RenderCommand::AddTile(c.x,c.z,true));

        move_point(map, c, &mut p, move_dir, clockwise);
        try_send!(render_sender, RenderCommand::AddTile(p.x,p.z,false));

        if clockwise {
            //4,8,C
            if p.z>=c.z && p.x+1==c.x {
                println!("48C");
                if can_move(map,c,Direction::Front) {
                    c.z+=1;
                    move_dir=Some(Direction::Front);
                }else {
                    match is_bridge(map, c, Direction::Front) {
                        Some(o) => {
                            p = o;

                            if can_move(map, c, Direction::Right) {
                                c.x += 1;
                                move_dir=Some(Direction::Right);
                            } else {
                                move_dir=None;
                            }
                        },
                        None => return ok!(None)
                    }
                }
            //D,E,F
            }else if p.x>=c.x && p.z==c.z+2 {
                println!("DEF");
                if can_move(map,c,Direction::Right) {
                    c.x+=1;
                    move_dir=Some(Direction::Right);
                }else{
                    match is_bridge(map,c,Direction::Right) {
                        Some(o) => {
                            p=o;

                            if can_move(map,c,Direction::Back) {
                                c.z-=1;
                                move_dir=Some(Direction::Back);
                            }else{
                                move_dir=None;
                            }
                        },
                        None => return ok!(None)
                    }
                }
            //3,7,B
            }else if p.z<c.z+2 && p.x==c.x+2 {
                println!("37B");
                if can_move(map,c,Direction::Back) {
                    c.z-=1;
                    move_dir=Some(Direction::Back);
                }else{
                    match is_bridge(map,c,Direction::Back) {
                        Some(o) => {
                            p=o;

                            if can_move(map,c,Direction::Left) {
                                c.x-=1;
                                move_dir=Some(Direction::Left);
                            }else{
                                move_dir=None;
                            }
                        },
                        None => return ok!(None)
                    }
                }
            //0,1,2
            }else if p.x<c.x+2 && p.z+1==c.z {
                println!("012");
                if can_move(map,c,Direction::Left) {
                    c.x-=1;
                    move_dir=Some(Direction::Left);
                }else{
                    match is_bridge(map,c,Direction::Left) {
                        Some(o) => {
                            p=o;

                            if can_move(map,c,Direction::Front) {
                                c.z+=1;
                                move_dir=Some(Direction::Front);
                            }else{
                                move_dir=None;
                            }
                        },
                        None => return ok!(None)
                    }
                }
            }

        }else{

        }
    }

    ok!(None)
}

fn move_point(map:&Map, c:Pos2D, p:&mut Pos2D, move_dir:Option<Direction>, clockwise:bool) {
    match move_dir {
        Some(Direction::Front) => {
            if p.z < MAP_SIZE1 && map.is_obstracle(p.x,p.z+1) {
                p.z+=1;
            }
        },
        Some(Direction::Right) => {
            if p.x < MAP_SIZE1 && map.is_obstracle(p.x+1,p.z) {
                p.x+=1;
            }
        },
        Some(Direction::Left) => {
            if p.x > 0 && map.is_obstracle(p.x-1,p.z) {
                p.x-=1;
            }
        },
        Some(Direction::Back) => {
            if p.z > 0 && map.is_obstracle(p.x,p.z-1) {
                p.z-=1;
            }
        },
        None => {},
    }

    if clockwise {
        for _ in 0..5 {
            println!("Move point {} {} {} {}",p.x,p.z, c.x, c.z);
            if p.x + 1 == c.x && p.z < c.z + 2 {
                println!("A");
                if p.z < MAP_SIZE1 && map.is_obstracle(p.x, p.z + 1) {
                    p.z += 1;
                } else {
                    return;
                }
            }

            if p.z == c.z + 2 && p.x < c.x + 2 {
                println!("B");
                if p.x < MAP_SIZE1 && map.is_obstracle(p.x + 1, p.z) {
                    p.x += 1;
                } else {
                    return;
                }
            }

            if p.x == c.x + 2 && p.z >= c.z {
                println!("C");
                if p.z > 0 && map.is_obstracle(p.x, p.z - 1) {
                    p.x -= 1;
                } else {
                    return;
                }
            }

            if p.z + 1 == c.z && p.x >= c.x {
                println!("D");
                if p.x > 0 && map.is_obstracle(p.x - 1, p.z) {
                    p.x -= 1;
                } else {
                    return;
                }
            }
        }
    }else{
        /*
        loop {

        }
        */
    }
}

fn can_move(map:&Map, c:Pos2D, dir:Direction) -> bool {
    match dir {
        Direction::Front =>
            c.z < MAP_SIZE2 && map.is_floor(c.x,c.z+2) && map.is_floor(c.x+1,c.z+2),
        Direction::Right =>
            c.x < MAP_SIZE2 && map.is_floor(c.x+2,c.z) && map.is_floor(c.x+2,c.z+1),
        Direction::Left =>
            c.x > 0 && map.is_floor(c.x-1,c.z) && map.is_floor(c.x-1,c.z+1),
        Direction::Back =>
            c.z > 0 && map.is_floor(c.x,c.z-1) && map.is_floor(c.x+1,c.z-1),
    }
}


fn is_bridge(map:&Map, c:Pos2D, dir:Direction) -> Option<Pos2D> {
    match dir {
        Direction::Front => {
            if c.z < MAP_SIZE2 {
                if map.is_floor(c.x,c.z+2) {
                    Some(Pos2D::new(c.x+1,c.z+2))
                }else if map.is_floor(c.x+1,c.z+2) {
                    Some(Pos2D::new(c.x,c.z+2))
                }else{
                    None
                }
            }else{
                None
            }
        },
        Direction::Right => {
            if c.x < MAP_SIZE2 {
                if map.is_floor(c.x+2,c.z) {
                    Some(Pos2D::new(c.x+2,c.z+1))
                }else if map.is_floor(c.x+2,c.z+1) {
                    Some(Pos2D::new(c.x+2,c.z))
                }else{
                    None
                }
            }else{
                None
            }
        },
        Direction::Left => {
            println!("LEFT {} {}",c.x,c.z);
            if c.x > 0 {
                if map.is_floor(c.x-1,c.z) {
                    Some(Pos2D::new(c.x-1,c.z+1))
                }else if map.is_floor(c.x-1,c.z+1) {
                    Some(Pos2D::new(c.x-1,c.z))
                }else{
                    None
                }
            }else{
                None
            }
        },
        Direction::Back => {
            println!("BACK {} {}",c.x,c.z);
            if c.z > 0 {
                if map.is_floor(c.x,c.z-1) {
                    println!("A");
                    Some(Pos2D::new(c.x+1,c.z-1))
                }else if map.is_floor(c.x+1,c.z-1){
                    println!("B");
                    Some(Pos2D::new(c.x,c.z-1))
                }else{
                    println!("C");
                    None
                }
            }else{
                None
            }
        }
    }
}

/*
fn hook(render_sender:&mut RenderSender, map:&mut Map, obstracle_pos:Pos2D, obstracle_dir:Direction, hook_dir:Direction, a:Pos2D, b:Pos2D, len:f32) -> Result<Option<Pos2D>,Error> {
    let ax=a.x as f32 + 1.0;
    let az=a.z as f32 + 1.0;
    let bx=b.x as f32 + 1.0;
    let bz=b.z as f32 + 1.0;

    let mut cur=obstracle_pos;
    let mut max_dist=0.0;
    let mut max_pos=None;

    let mark=map.get_mark();
    //map.mark(obstracle_pos.0, obstracle_pos.1,mark);

    match obstracle_dir {
        Direction::Front => {
            if !map.is_floor(obstracle_pos.0,obstracle_pos.1+2) {
                map.mark(obstracle_pos.0,obstracle_pos.1+2,mark);
            }
            if !map.is_floor(obstracle_pos.0+1,obstracle_pos.1+2) {
                map.mark(obstracle_pos.0+1,obstracle_pos.1+2,mark);
            }

            match hook_dir {
                Direction::Left => {
                    if obstracle_pos.1 > 0 {
                        if !map.is_floor(obstracle_pos.0,obstracle_pos.1-1) {
                            map.mark(obstracle_pos.0+1,obstracle_pos.1-1,mark);
                        }
                    }
                },
                Direction::Right => {
                    if obstracle_pos.1+1 < MAP_SIZE as u32 {
                        if !map.is_floor(obstracle_pos.0,obstracle_pos.1+1) {
                            map.mark(obstracle_pos.0+1,obstracle_pos.1+1,mark);
                        }
                    }
                },
                _ => unreachable!()
            }
        }
        _ => {},
    }

    loop {
        try_send!(render_sender, RenderCommand::AddTile(cur.0,cur.1));
        thread::sleep_ms(1000);
        map.mark(cur.0, cur.1, mark);
        mark_around(map, cur, mark);
        //как-то надо задать направление.. а что если просто вызвать эту функцию дважды с одним mark?
        println!("{} {}", cur.0, cur.1);

        //Horisontal
        if cur.1 + 2 < MAP_SIZE as u32 && (!map.is_floor(cur.0, cur.1 + 2) && map.is_marked(cur.0, cur.1 + 2, mark) || !map.is_floor(cur.0 + 1, cur.1 + 2) && map.is_marked(cur.0 + 1, cur.1 + 2, mark)) {
            println!("a");
            if cur.0 > 0 && map.is_floor(cur.0 - 1, cur.1) && !map.is_marked(cur.0 - 1, cur.1, mark) {
                cur.0 -= 1;
                //mark,
                //calc dist
                //mark around
                continue;
            }

            if cur.0 < MAP_SIZE as u32 - 2 && map.is_floor(cur.0 + 1, cur.1) && !map.is_marked(cur.0 + 1, cur.1, mark) {
                cur.0 += 1;
                continue;
            }
        }

        if cur.1 > 0 && (!map.is_floor(cur.0, cur.1 - 1) && map.is_marked(cur.0, cur.1 - 1, mark) || !map.is_floor(cur.0 + 1, cur.1 - 1) && map.is_marked(cur.0 + 1, cur.1 - 1, mark)) {
            println!("b");
            if cur.0 > 0 && map.is_floor(cur.0 - 1, cur.1) && !map.is_marked(cur.0 - 1, cur.1, mark) {
                cur.0 -= 1;
                continue;
            }

            if cur.0 < MAP_SIZE as u32 - 2 && map.is_floor(cur.0 + 1, cur.1) && !map.is_marked(cur.0 + 1, cur.1, mark) {
                cur.0 += 1;
                continue;
            }
        }

        //Vertical
        if cur.0 + 2 < MAP_SIZE as u32 && (!map.is_floor(cur.0 + 2, cur.1) && map.is_marked(cur.0 + 2, cur.1, mark) || !map.is_floor(cur.0 + 2, cur.1 + 1) && map.is_marked(cur.0 + 2, cur.1 + 1, mark)) {
            println!("c");
            if cur.1 > 0 && map.is_floor(cur.0, cur.1 - 1) && !map.is_marked(cur.0, cur.1 - 1, mark) {
                cur.1 -= 1;
                //mark,
                //calc dist
                //mark around
                continue;
            }

            if cur.1 < MAP_SIZE as u32 - 2 && map.is_floor(cur.0, cur.1 + 1) && !map.is_marked(cur.0, cur.1 + 1, mark) {
                cur.1 += 1;
                continue;
            }
        }

        if cur.0 > 0 && (!map.is_floor(cur.0 - 1, cur.1) && map.is_marked(cur.0 - 1, cur.1, mark) || !map.is_floor(cur.0 - 1, cur.1 + 1) && map.is_marked(cur.0 - 1, cur.1 + 1, mark)) {
            println!("d");
            if cur.1 > 0 && map.is_floor(cur.0, cur.1 - 1) && !map.is_marked(cur.0, cur.1 - 1, mark) {
                cur.1 -= 1;
                //mark,
                //calc dist
                //mark around
                continue;
            }

            if cur.1 < MAP_SIZE as u32 - 2 && map.is_floor(cur.0, cur.1 + 1) && !map.is_marked(cur.0, cur.1 + 1, mark) {
                cur.1 += 1;
                continue;
            }
        }

        //Corner +1 +1
        if cur.0 < MAP_SIZE as u32 - 2 && cur.1 < MAP_SIZE as u32 - 2 && map.is_marked(cur.0 + 2, cur.1 + 2, mark) || !map.is_floor(cur.0 + 2, cur.1 + 2){
            println!("ADD");
            if !map.is_floor(cur.0, cur.1 + 2) && !map.is_marked(cur.0, cur.1 + 2, mark) {//bridge
                println!("A");
                map.mark(cur.0, cur.1 + 2, mark);

                if cur.0 >0 && map.is_floor(cur.0 - 1, cur.1) && !map.is_marked(cur.0 - 1, cur.1, mark) {
                    cur.0 -= 1;
                    continue;
                }
            }

            if cur.0 < MAP_SIZE as u32 - 2 && !map.is_floor(cur.0+2, cur.1 + 1) {//gate

            }

            if cur.1 < MAP_SIZE as u32 - 2 && map.is_floor(cur.0, cur.1 + 2) && !map.is_marked(cur.0, cur.1 + 2, mark) {
                cur.1 += 1;
                continue;
            }
        }

        break;
    }

    ok!(max_pos)
}

fn mark_around(map:&mut Map, cur:Pos2D, mark:u32) {
    //loop {
        if cur.1 > 0 {
            mark_obstracle(map, (cur.0, cur.1 - 1), mark);
            mark_obstracle(map, (cur.0 + 1, cur.1 - 1), mark);
            //continue;
        }

        if cur.0 > 0 {
            mark_obstracle(map, (cur.0 - 1, cur.1), mark);
            mark_obstracle(map, (cur.0 - 1, cur.1 + 1), mark);
            //continue;
        }

        if cur.1 < MAP_SIZE as u32 - 2 {
            mark_obstracle(map, (cur.0, cur.1 + 2), mark);
            mark_obstracle(map, (cur.0 + 1, cur.1 + 2), mark);
            //continue;
        }

        if cur.0 < MAP_SIZE as u32 - 2 {
            mark_obstracle(map, (cur.0 + 2, cur.1), mark);
            mark_obstracle(map, (cur.0 + 2, cur.1 + 1), mark);
            //continue;
        }

        if cur.1 > 0 && cur.0 > 0 {
            mark_obstracle(map, (cur.0 - 1, cur.1 - 1), mark);
            //continue;
        }

        if cur.1 > 0 && cur.0 < MAP_SIZE as u32 - 2 {
            mark_obstracle(map, (cur.0 + 2, cur.1 - 1), mark);
            //continue;
        }

        if cur.1 < MAP_SIZE as u32 - 2 && cur.0 > 0 {
            mark_obstracle(map, (cur.0 - 1, cur.1 + 1), mark);
            //continue;
        }

        if cur.1 < MAP_SIZE as u32 - 2 && cur.0 < MAP_SIZE as u32 - 2 {
            mark_obstracle(map, (cur.0 + 2, cur.1 + 1), mark);
            //continue;
        }

        //break;
    //}
}

fn mark_obstracle(map:&mut Map, pos:Pos2D, mark:u32) {
    if map.is_floor(pos.0,pos.1) || map.is_marked(pos.0,pos.1, mark) {
        return;
    }

    if pos.0 == 8 {
        println!("H!");
    }

    if pos.1>0 {
        println!("a!");
        if !map.is_floor(pos.0,pos.1-1) && map.is_marked(pos.0,pos.1-1,mark) {
            map.mark(pos.0, pos.1, mark);
            println!("Marked a!");
            return;
        }
    }

    if pos.0>0 {
        println!("b!");
        if !map.is_floor(pos.0-1,pos.1) && map.is_marked(pos.0-1,pos.1,mark) {
            map.mark(pos.0, pos.1, mark);
            println!("Marked b!");
            return;
        }
    }

    if pos.1<MAP_SIZE as u32 -1 {
        println!("c!");
        if !map.is_floor(pos.0,pos.1+1) && map.is_marked(pos.0,pos.1+1,mark) {
            map.mark(pos.0, pos.1, mark);
            println!("Marked c!");
            return;
        }
    }

    if pos.0<MAP_SIZE as u32 -1 {
        println!("d!");
        if !map.is_floor(pos.0+1,pos.1) && map.is_marked(pos.0+1,pos.1,mark) {
            map.mark(pos.0, pos.1, mark);
            println!("Marked d!");
            return;
        }
    }
}
*/

/*
fn hook(render_sender:&mut RenderSender, map:&mut Map, mut prev:Pos2D, obstracle_pos:Pos2D, a:Pos2D, b:Pos2D, dir:Direction, len:f32) -> Result<Option<Pos2D>,Error> {
    let ax=a.x as f32 + 1.0;
    let az=a.z as f32 + 1.0;
    let bx=b.x as f32 + 1.0;
    let bz=b.z as f32 + 1.0;

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
                /*
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
                */

                let sel=match variants[2] {
                    Some((pos,dist)) =>
                        Some((pos,dist)),
                    None => {
                        match variants[0] {
                            Some((pos,dist)) =>
                                Some((pos,dist)),
                            None => {
                                match variants[1] {
                                    Some((pos,dist)) =>
                                        Some((pos,dist)),
                                    None => {
                                        match variants[3] {
                                            Some((pos, dist)) =>
                                                Some((pos, dist)),
                                            None => None,
                                        }
                                    }
                                }
                            }
                        }
                    }
                };

                match sel {
                    Some((new_pos,new_dist)) => {
                        println!("New Pos {} {}",new_pos.0,new_pos.1);

                        if new_dist>max_dist {
                            max_pos=Some(new_pos);
                            max_dist=new_dist;
                        }

                        if new_dist<0.5 {
                            //TODO:совершила круг
                            //break;
                        }

                        prev=cur;
                        cur=new_pos;

                        try_send!(render_sender, RenderCommand::AddTile(cur.0,cur.1));
                        thread::sleep_ms(1000);
                    },
                    None => {
                        let tmp=cur;
                        cur=prev;
                        prev=cur;
                    }
                }
            }
        },
        _ => {}
    }

    ok!(max_pos)
}
*/



pub fn trace_line(traces:&mut TracePool, storage:&Storage, render_sender:&mut RenderSender, map:&mut Map, a:Pos2D, b:Pos2D, trace_id:TraceID) -> Result<(),Error> {
    let obstracle=find_obstracle(render_sender, map, a,b)?;
    let (dir,_,len,_) = calc_trace(a,b);

    match obstracle {
        Some(obstracle_pos) => {
            try_send!(render_sender, RenderCommand::SetTraceColor(trace_id,RED));

            let hooks_pos=match dir {
                Direction::Front => {
                    println!("Obstracle {} {}",obstracle_pos.x,obstracle_pos.z);

                    (
                        hook(render_sender, map, obstracle_pos, Direction::Front, true, a, b, len)?,
                        //hook(render_sender, map, obstracle_pos, Direction::Front, Direction::Left, a, b, len)?,
                        //hook(render_sender, map,(obstracle_pos.0-1,obstracle_pos.1), obstracle_pos,  a, b, dir, len)?,
                        //hook(render_sender, map,obstracle_pos, (obstracle_pos.0+1,obstracle_pos.1), a, b, dir, len)?
                        None
                    )
                },
                Direction::Back => {
                    (
                        //hook(render_sender, map,obstracle_pos, (obstracle_pos.0+1,obstracle_pos.1), a, b, dir, len)?,
                        //hook(render_sender, map,obstracle_pos, (obstracle_pos.0-1,obstracle_pos.1), a, b, dir, len)?
                        None,None
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
/*
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
            */
        },
        None => {
            try_send!(render_sender, RenderCommand::SetTraceColor(trace_id,GREEN));
        },
    }

    //try_send!(render_sender, RenderCommand::ClearTiles);

    ok!()
}