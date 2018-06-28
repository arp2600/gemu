use std::thread::sleep;
use std::time::{Duration, SystemTime};

pub struct FrameTimer {
    start: SystemTime,
    frame_time: u32,
}

impl FrameTimer {
    pub fn new(frame_time_nanos: u32) -> FrameTimer {
        FrameTimer {
            start: SystemTime::now(),
            frame_time: frame_time_nanos,
        }
    }

    pub fn sleep_till_end_of_frame(&self) {
        let frame_time = Duration::new(0, self.frame_time);
        let sleep_time = match self.start.elapsed() {
            Ok(x) => if frame_time > x {
                frame_time - x
            } else {
                Duration::new(0, 0)
            },
            Err(_) => Duration::new(0, 0),
        };
        sleep(sleep_time);
    }
}
