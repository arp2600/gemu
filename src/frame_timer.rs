use std::thread::sleep;
use std::time::{Duration, SystemTime};

pub struct FrameTimer {
    next_frame: SystemTime,
    frame_duration: Duration,
}

impl FrameTimer {
    pub fn new(fps: f32) -> FrameTimer {
        let frame_duration = {
            let nanos = (1_000_000_000.0 / fps) as u32;
            Duration::new(0, nanos)
        };

        let next_frame = SystemTime::now() + frame_duration;

        FrameTimer {
            next_frame,
            frame_duration,
        }
    }

    pub fn sleep_then_update(&mut self) {
        let now = SystemTime::now();
        if now < self.next_frame {
            sleep(self.next_frame.duration_since(now).unwrap());
        }
        self.next_frame += self.frame_duration;
    }
}
