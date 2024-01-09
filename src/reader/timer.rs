use std::sync::mpsc::{channel, Receiver};
use async_std::task;
use async_std::task::sleep;
use chrono::Duration;
use async_std::task::JoinHandle;
use tokio::select;
use crate::error::Error;
use crate::reader::read_task::ReadTask;

pub struct Timer {
    timer_task: JoinHandle<()>,
    end_timer: Option<Receiver<()>>,
    wake_up: Receiver<()>,
}

impl Timer {
    pub fn init(total_time: Option<Duration>, wake_up_interval: Duration) -> Timer{
        let (timer_sender, timer_receiver) = match total_time {
            Some(_) => {let (s,r) = channel::<()>(); (Some(s),Some(r))},
            None => (None,None)
        };
        let (wake_up_sender, wake_up_receiver) = channel::<()>();
        let task = task::spawn(task::spawn(async move {
            let timer = if let Some(total_time) = total_time {
                let timer_sender = timer_sender.unwrap();
                 Some(async move {
                     sleep(total_time.to_std().unwrap()).await;
                     if let Err(e) = timer_sender.send(()) {
                         println!("timer end: closed channel")
                         // Trigger by closed channel
                     }
                })
            }else {
                None
            };
            let wake_up = async {
                loop{
                    sleep(wake_up_interval.to_std().unwrap()).await;
                    if let Err(e) = wake_up_sender.send(()) {
                        // Trigger by closed channel
                    }
                }
            };
            if timer.is_none(){
                wake_up.await;
            }else{
                select!(_ = timer.unwrap() => {}, _ = wake_up => {});
            }
        }));

        Timer {
            timer_task: task,
            end_timer: timer_receiver,
            wake_up: wake_up_receiver,
        }
    }

    pub fn destroy(self){
        drop(async move{
            self.timer_task.cancel().await.unwrap()
        })
    }

    pub fn has_end(&self) -> bool {
        self.end_timer.is_some()
    }

    pub fn wait_end(self) -> Result<(),Error>{
        if !self.has_end() {
            return Err(Error::TimerError("wait_end: There are no end".to_string()));
        }
        if let Err(e) = self.end_timer.unwrap().recv() {
            return Err(Error::ReceiveError(format!("Timer::wait_end : {e}")))
        }
        Ok(())
    }

    pub fn wait_wake_up(&self) -> Result<(),Error>{
        if let Err(e) = self.wake_up.recv() {
            return Err(Error::ReceiveError(format!("Timer::wait_end : {e}")))
        }
        Ok(())
    }

    pub fn wait_timer_or_task(self, read_task: ReadTask) -> Result<(),Error>{
        if !self.has_end() {
            return Err(Error::IllegalState("timer::wait_timer_or_task: Timer has no end".to_string()))
        }

        let (sender,join_receiver) = channel::<()>();
        let exec_sender = sender.clone();

        task::spawn(async move {
            let timer_end = async move {
                self.wait_end().unwrap();
                sender.send(()).unwrap()
            };

            let exec_task = async move {
                read_task.wait_end().unwrap();
                exec_sender.send(()).unwrap()
            };
            select!(_ = timer_end => {},_ = exec_task => {});
        });
        if let Err(e) = join_receiver.recv() {
            return Err(Error::ReceiveError(format!("Timer::wait_timer_or_end : {e}")))
        }
        Ok(())
    }
}