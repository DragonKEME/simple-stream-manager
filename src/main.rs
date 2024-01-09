mod reader;
mod queue;
mod error;
mod test;

use chrono::{Local,Duration};
use crate::queue::queue::set_queue;
use crate::queue::video_time::VideoTime;
use crate::reader::task_manager::TaskManager;


const INTERSTICE: &str = "/home/dkeme/Vidéos/streamtest/placeholder.h264";

const PATH: &str = "/home/dkeme/Vidéos/streamtest/queue/";

const PIPE_PATH: &str = "/home/dkeme/Vidéos/streamtest/stream";

#[tokio::main]
async fn main() {

    let interstitial = VideoTime {
        video: INTERSTICE.to_string(),
        time: None
    };

    //let mut read_task = ReadTask::make_interstice(padding_content).unwrap();

    println!("start make queue");
    let local = Local::now();
    let queue = vec![
        VideoTime{
            // New york
            video: PATH.to_string() + "pexels-corentin-jacquemaire-19220171-1080p.mp4.h264",
            time: Some(local + Duration::seconds(20))
        },
        VideoTime{
            // Baleine
            video: PATH.to_string() + "pexels-kammeran-gonzalezkeola-17823998-1080p.mp4.h264",
            time: None
        },
        VideoTime{
            // Volcan
            video: PATH.to_string() + "pexels-sooin-kim-19148208-1080p.mp4.h264",
            time: Some(local + Duration::seconds(65))
        },
        VideoTime{
            // Cascade
            video: PATH.to_string() + "pexels-visual-soundscapes-19264911-1080p.mp4.h264",
            time: None
        },
        VideoTime{
            //
            video: PATH.to_string() + "production_id-4613097-1080p.mp4.h264",
            time: Some(local + Duration::seconds(123))
        },
    ];
    set_queue(queue);
    match job(interstitial).await {
        Ok(()) => (),
        Err(e) => print!("{e}")
    }

}
pub async fn job(interstitial: VideoTime) -> Result<(), error::Error>{
    let mut tm = TaskManager::init(interstitial)?;
    tm = tm.lunch()?;
    println!("task lunched");
    loop {
        tm = tm.wait_next_task()?;
    }
    //Ok(())
}
