use std::fs;
use std::ops::Range;
use async_std::task::{JoinHandle};
use std::sync::mpsc::{channel, Sender, Receiver, TryRecvError};
use async_std::task;
use crate::PIPE_PATH;
use crate::error::Error;
use crate::queue::video_time::VideoTime;



pub struct ReadTask {
    pub task: JoinHandle<()>,
    begin_sender: Sender<()>,                       // Task begin after that
    end_receiver: Option<Receiver<()>>,         // Task send here when end (option cause some task never end)
    destroy_sender: Sender<()>,
    launch_state: bool
}


impl ReadTask{
    pub fn make_task(video: VideoTime) -> ReadTask{
        let (begin_sender, begin_receiver) = channel::<()>();
        let (end_sender, end_receiver) = channel::<()>();
        let (destroy_sender, destroy_receiver) = channel::<()>();
        let task: JoinHandle<()> = task::spawn(async move {
            read(video, begin_receiver, end_sender, destroy_receiver);
        });
        ReadTask {
            task,
            begin_sender,
            end_receiver: Some(end_receiver),
            destroy_sender,
            launch_state: false
        }
    }

    pub fn make_interstice(video_time: VideoTime) -> Result<ReadTask, Error>{
        //Load content
        let (begin_sender, begin_receiver) = channel();
        let (destroy_sender, destroy_receiver) = channel();
        let content = match fs::read(video_time.clone().video) {
            Ok(content) => content,
            Err(_) => return Err(Error::BuildInterstice("Video not found".to_string()))
        };

        //Next task
        let task: JoinHandle<()> = task::spawn(async move {
            // Cut content in some part
            let content_length = content.len();
            let max_slice = content_length/100;
            let mut running_range: Vec<Range<usize>> = vec![];
            for i in 0..max_slice {
                running_range.insert(i, i*100..(i+1)*100)
            }
            running_range.insert(max_slice,max_slice*100..content_length);

            // Wait launch
            if let Err(e) = begin_receiver.recv() { println!("{e}"); return  }

            loop {
                for i in running_range.clone() {
                    fs::write(PIPE_PATH,&content[i]).unwrap();
                    match destroy_receiver.try_recv() {
                        Ok(_) | Err(TryRecvError::Disconnected) => {
                            println!("task terminated");
                            return;
                        }
                        Err(TryRecvError::Empty) => {}
                    }
                }
            }
        });

        Ok(ReadTask {
            begin_sender,
            task,
            end_receiver: None,
            destroy_sender,
            launch_state: false
        })
    }

    pub fn destroy(self){
        if let Err(e) = self.destroy_sender.send(()){
            println!("destroy Error: {e}");
        }
    }

    pub fn launch(&mut self) -> Result<(),Error>{
        if let Err(e) = self.begin_sender.send(()) {
            return Err(Error::SendError(e.to_string()))
        }
        self.launch_state = true;
        Ok(())
    }

    pub fn has_end(&self) -> bool {
        self.end_receiver.is_some()
    }

    pub fn wait_end(self) -> Result<(),Error>{
        if self.end_receiver.is_none(){
            return Err(Error::IllegalState("read_task::wait_end: There no end receiver".to_string()));
        }
        if let Err(e) = self.end_receiver.unwrap().recv(){
            return Err(Error::ReceiveError(format!("wait_current: {e}")));
        }
        Ok(())
    }
}
fn read(video_time: VideoTime, starter: Receiver<()>, next_sender: Sender<()>, destroy_receiver: Receiver<()>){
    //Load content
    // TODO: remove unwrap or panic
    let content = fs::read(video_time.video).unwrap();

    // Cut content in some part
    let content_length = content.len();
    let max_slice = content_length/100;
    let mut running_range: Vec<Range<usize>> = vec![];
    for i in 0..max_slice {
        running_range.insert(i, i*100..(i+1)*100)
    }
    running_range.insert(max_slice,max_slice*100..content_length);

    //Wait trigger
    starter.recv().unwrap();
    //Stream
    for i in running_range.clone() {
        fs::write(PIPE_PATH,&content[i]).unwrap();
        match destroy_receiver.try_recv() {
            Ok(_) | Err(TryRecvError::Disconnected) => {
                println!("task terminated");
                return;
            }
            Err(TryRecvError::Empty) => {}
        }
    }
    next_sender.send(()).unwrap()
}




