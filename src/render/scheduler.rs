
use types::*;

pub struct Scheduler {
    normal_fi:Duration,
    normal_fps:u32,

    pub frame_begin:Time,
    pub poll_window_events_end:Time,
    pub handle_render_commands_end:Time,
    pub handle_storage_commands_end:Time,
    pub rendering_end:Time,
    pub frame_end:Time,

    pub plan_storage_commands_handling_i:Duration,
    pub plan_fi:Duration
}

impl Scheduler {
    pub fn new(fps:u32) -> Self {
        let normal_fi=Duration::new(1,0)/fps;

        Scheduler{
            normal_fi,
            normal_fps:fps,

            frame_begin:Time::now(),
            poll_window_events_end:Time::now(),
            handle_render_commands_end:Time::now(),
            handle_storage_commands_end:Time::now(),
            rendering_end:Time::now(),
            frame_end:Time::now(),

            plan_storage_commands_handling_i:Duration::new(0,10_000_000),
            plan_fi:Duration::new(0,20_000_000)
        }
    }

    pub fn make_plan(&mut self) -> Option<Duration>{
        let frame_i=self.frame_end.duration_since(self.frame_begin).unwrap();
        let rendering_i=self.rendering_end.duration_since(self.handle_storage_commands_end).unwrap();

        self.plan_storage_commands_handling_i=Duration::new(0,10_000_000);
        self.plan_fi=Duration::new(0,20_000_000);

        if frame_i > self.normal_fi {
            None
        }else{
            Some(self.normal_fi - frame_i)
        }
    }
}

/*
pub struct Scheduler {
    normal_fi:Duration,
    normal_fps:usize,

    pub rendering_i:Duration,
    pub loading_resources_i:Duration,
    pub animation_i:Duration,

    plan_fi:Duration,
    plan_loading_resources_fi:Duration
}


impl Scheduler {
    pub fn new(fps:usize) -> Self {
        let normal_fi:Duration::new(1,0)/fps;

        Scheduler {
            normal_fi:normal_fi,
            normal_fps:fps,

            rendering_i:Duration::new(0,0),
            loading_resources_i:Duration::new(0,0),
            animation_i:Duration::new(0,0),

            plan_fi:normal_fi,
            plan_loading_resources_fi:normal_fi*0.5,
        }
    }

    pub fn make_plan(&mut self) -> Self {
        use std::cmp::max;

        let render_fi=self.loading_resources_i+self.rendering_i;
        let max_fi=max(render_fi, animation_i);

        if max_fi > self.normal_fi {//Запаздывает
            if render_fi>self.normal_fi && self.animation_i>self.normal_fi {
                let plan_loading_resources_i=render_fi*0.1;
                let plan_render_fi=plan_loading_resources_i+self.rendering_i;

                if plan_render_fi > self.animation_i {//Увы, loading_resources должен занимать 10% времени кадра
                    self.plan_fi=plan_render_fi;
                    self.plan_loading_resources_fi=plan_loading_resources_i;
                }else{
                    self.plan_fi=self.animation_i;
                    self.plan_loading_resources_fi=self.animation_i-self.rendering_i;
                }
            }else if render_fi>self.normal_fi {
                let plan_loading_resources_i=render_fi*0.1;
                let plan_render_fi=plan_loading_resources_i+self.rendering_i;

                if plan_render_fi > self.normal_fi {  //Увы, loading_resources должен занимать 10% времени кадра
                    self.plan_fi = plan_render_fi;
                    self.plan_loading_resources_fi = plan_loading_resources_i;
                }else{//можно добиться нормальных показателей
                    self.plan_fi=self.normal_fi;
                    self.plan_loading_resources_fi=self.normal_fi-self.rendering_i;
                }
            }else if self.animation_i>self.normal_fi {
                self.plan_fi=self.animation_i;
                self.plan_loading_resources_fi=self.animation_i-self.rendering_i;
            }
        }else{
            self.plan_fi=self.normal_fi;
            self.plan_loading_resources_fi=self.normal_fi-self.rendering_i;
        }
    }
}

*/