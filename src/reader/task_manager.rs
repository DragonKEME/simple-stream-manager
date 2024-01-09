use chrono::{Local, Duration};
use crate::queue::video_time::VideoTime;
use crate::queue::queue::{pop_next, schedule_next};
use crate::reader::read_task::ReadTask;
use crate::reader::timer::Timer;
use crate::error::Error;


pub struct TaskManager {
    current_task: ReadTask,
    next_task: Option<ReadTask>,
    interstitial: ReadTask,
    interstitial_video: VideoTime,
    timer: Option<Timer>
}

impl TaskManager{
    pub fn init(interstitial_video: VideoTime) -> Result<TaskManager, Error> {
        //Check if there is a video in queue
        let (mut first_task,timer) = TaskManager::check_time_and_init_video_timer(false);

        //init interstial
        let interstitial = ReadTask::make_interstice(interstitial_video.clone())?;

        let next_task = if first_task.is_some() && timer.is_some() {
            //If timer exist is for a next launch
            let ft = first_task;
            first_task = None;
            ft
        }else{
            None
        };

        let current_task = match first_task {
            Some(ft) => ft,
            None => ReadTask::make_interstice(interstitial_video.clone())?
        };

        Ok(TaskManager {
            current_task,
            next_task,
            interstitial,
            interstitial_video,
            timer
        })
    }

    pub fn lunch(mut self) -> Result<TaskManager, Error> {

        if let Err(e) = self.current_task.launch() {
            return Err(Error::SendError(e.to_string()));
        }

        if self.next_task.is_none() && self.timer.is_none(){
            let has_end = self.current_task.has_end();
            (self.next_task,self.timer) = TaskManager::check_time_and_init_video_timer(has_end);
            // Direct next and current has no end
            if self.next_task.is_some() && self.timer.is_none() && !has_end {
                self = self.force_next()?;
            }
        }

        Ok(self)
    }

    fn force_next(mut self) -> Result<TaskManager,Error>{
        if self.next_task.is_none(){
            return Err(Error::IllegalState("Lunch_next: Next doesn't exist".to_string()))
        }
        let has_next = self.current_task.has_end();
        self.current_task.destroy();
        self.current_task = self.next_task.unwrap();
        (self.next_task,self.timer) = TaskManager::check_time_and_init_video_timer(has_next);
        Ok(self)
    }

    fn check_time_and_init_video_timer(current_has_end: bool) -> (Option<ReadTask>,Option<Timer>) {
        //Possible state:
        // None, None -> has_end, no_next
        // None, Some -> No_end, no_next or //, indirect_next
        // Some, None -> //, direct_next
        // Some, Some -> //, Scheduled_next
        let scheduled_time = match schedule_next() {
            //Video in queue exist
            Some(s) => s,
            //No video in queue
            None => {
                return if current_has_end {
                    // Current has end so no timer
                    (None, None)
                } else {
                    // Current as no end so timer wake-up
                    (None, Some(Timer::init(None, Duration::seconds(1))))
                }
            }
        };
        //schedule == none prepare immediately task (no timer required)
        if scheduled_time.is_none() {
            return (Some(ReadTask::make_task(pop_next().unwrap())), None) // At this point video must exist
        }
        let time_before_next_video = scheduled_time.unwrap() - Local::now();

        //if duration < 10 second prepare video_task
        let read_task = if time_before_next_video < Duration::seconds(10) {
            Some(ReadTask::make_task(pop_next().unwrap()))
        }else {
            None
        };
        //Prepare timer anytime when next video exist with scheduled
        let timer = Timer::init(Some(time_before_next_video),Duration::seconds(1));

        (read_task, Some(timer))
    }

    pub fn wait_next_task(mut self) -> Result<TaskManager, Error>{
        println!("Wait next task:\n{}", self.dump_state());
        if self.current_task.has_end() {
            // Current task has end (eC)
            if self.timer.is_some() {
                // and there are a timer (eC.T = W(c+t))
                self = self.wait_timer_or_current()?;
            }else {
                // No timer (eC.nT = Wc)
                self = self.wait_current()?;
            }
        }else if self.timer.is_some() {
            // Current task has no end and There are a timer (neC.T)
            if self.next_task.is_some() {
                // Next task exist (neC.T.N = Wt)
                self = self.wait_timer()?;
            }else {
                // Next task doesn't exist (neC.T.nN = Wu)
                self = self.wait_wake_up()?;
            }
        }else {
            // Special case supposed to not exist
            if self.next_task.is_some() {
                self.current_task = self.next_task.unwrap();
                self.current_task.launch()?;
            }
            (self.next_task, self.timer) = TaskManager::check_time_and_init_video_timer(false);
            print!("Warning: No video_ending and no timer");
        }
        Ok(self)
    }

    fn wait_timer_or_current(mut self) -> Result<TaskManager, Error>{
        println!("Debug: Wait_timer_or_current");
        // Wait timer and current in same time

        if !self.current_task.has_end() {
            return Err(Error::IllegalState("wait_timer_or_current: current has no end".to_string()));
        }
        if self.timer.is_none() {
            return Err(Error::IllegalState("wait_timer_or_current: There are no timer".to_string()))
        }

        //Wait timer or current
        self.timer.unwrap().wait_timer_or_task(self.current_task)?;

        match self.next_task {
            // Next exist launch next
            Some(task) => {
                self.current_task = task;
                self.current_task.launch()?;
                (self.next_task, self.timer) = TaskManager::check_time_and_init_video_timer(true);
            }
            None => {
                // Get new video
                (self.next_task, self.timer) = TaskManager::check_time_and_init_video_timer(false);

                if self.next_task.is_some() && self.timer.is_none() {
                    // Next video is direct launch next
                    self.current_task = self.next_task.unwrap();
                    self.current_task.launch()?;
                    self.next_task = None;
                } else {
                    // There are no direct job launch interstice
                    self.current_task = self.interstitial;
                    self.current_task.launch()?;
                    self.interstitial = ReadTask::make_interstice(self.interstitial_video.clone())?;
                }
            }
        }

        Ok(self)
    }

    fn wait_timer(mut self) -> Result<TaskManager,Error>{
        println!("Debug: Wait_timer");
        // Wait timer only - current has no_end and next exist
        if self.current_task.has_end() {
            return Err(Error::IllegalState("wait_timer: current has end".to_string()));
        }

        if self.timer.is_none() {
            return Err(Error::IllegalState("wait_timer: There are no timer".to_string()))
        }

        if self.next_task.is_none() {
            return Err(Error::IllegalState("wait_timer: There are no next task".to_string()))
        }

        // Wait timer
        self.timer.unwrap().wait_end()?;
        // Destroy current task
        self.current_task.destroy();

        // Next task exist
        self.current_task = self.next_task.unwrap();
        self.current_task.launch()?;
        (self.next_task, self.timer) = TaskManager::check_time_and_init_video_timer(true);

        Ok(self)
    }

    fn wait_wake_up(mut self) -> Result<TaskManager ,Error>{
        println!("Debug: Wait_wake_up");
        // Wake up - Only timer exist
        if self.timer.is_none() {
            return Err(Error::IllegalState("wait_wake_up: there no timer".to_string()));
        }

        self.timer.as_ref().unwrap().wait_wake_up()?;

        if let Some(st) = schedule_next() {
            if st.is_none() || st.unwrap() - Local::now() < Duration::seconds(10){
                // Next video is ready
                self.timer.unwrap().destroy();
                (self.next_task,self.timer) = TaskManager::check_time_and_init_video_timer(false)
            }
        }

        Ok(self)
    }

    fn wait_current(mut self) -> Result<TaskManager,Error>{
        println!("Debug: Wait_current");
        // Wait current only - timer does not exist at this point
        if self.timer.is_some() {
            return Err(Error::IllegalState("wait_current: There are a timer in wait current only".to_string()));
        }

        //Wait end
        self.current_task.wait_end()?;

        match self.next_task {
            // Next exist launch next
            Some(task) => {
                self.current_task = task;
                self.current_task.launch()?;
                (self.next_task, self.timer) = TaskManager::check_time_and_init_video_timer(true);
            }
            None => {
                // Get new video
                (self.next_task, self.timer) = TaskManager::check_time_and_init_video_timer(false);

                if self.next_task.is_some() && self.timer.is_none() {
                    // Next video is direct launch next
                    self.current_task = self.next_task.unwrap();
                    self.current_task.launch()?;
                    self.next_task = None;
                } else {
                    // There are no direct job launch interstice
                    self.current_task = self.interstitial;
                    self.current_task.launch()?;
                    self.interstitial = ReadTask::make_interstice(self.interstitial_video.clone())?;
                }
            }
        }

        Ok(self)
    }

    pub fn dump_state(&self) -> String{
        let ec = if self.current_task.has_end(){
            "1"
        }else {
            "0"
        };
        let n = if self.next_task.is_some(){
            "1"
        }else {
            "0"
        };
        let t = if self.timer.is_some(){
            "1"
        }else {
            "0"
        };

        format!("| Ec | n | t |\n| {ec}  | {n} | {t} |")
    }


}
