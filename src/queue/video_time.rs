use chrono::{DateTime, Local};

pub type ScheduledTime = Option<DateTime<Local>>;
#[derive(Clone)]
pub struct VideoTime {
    pub video: String,
    pub time: ScheduledTime
}