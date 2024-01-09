use once_cell::sync::OnceCell;
use std::sync::Mutex;
use crate::queue::video_time::{ScheduledTime, VideoTime};

static QUEUE: OnceCell<Mutex<Vec<VideoTime>>> = OnceCell::new();

fn ensure_queue() -> &'static Mutex<Vec<VideoTime>> {
    QUEUE.get_or_init(|| Mutex::new(Vec::new()))
}

pub fn get_queue() -> Vec<VideoTime> {
    ensure_queue().lock().unwrap().clone()
}

pub fn set_queue(queue: Vec<VideoTime>) {
    *ensure_queue().lock().unwrap() = queue;
}

pub fn pop_next() -> Option<VideoTime>{
    let mut video = ensure_queue().lock().unwrap();
    if video.is_empty(){
        return None
    }
    Some(video.remove(0))
}

pub fn schedule_next() -> Option<ScheduledTime>{
    let video = ensure_queue().lock().unwrap();
    if video.is_empty(){
        return None
    }
    Some(video[0].time)
}
