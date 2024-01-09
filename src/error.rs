use std::{error, fmt};

#[derive(Debug)]
pub enum Error {
    BuildInterstice(String),
    BuildVideoReadTask(String),
    SendError(String),
    ReceiveError(String),
    IllegalState(String),
    TimerError(String),
    NoTask,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::BuildInterstice(s) => write!(f, "Unable to build interstice: {s}"),
            Error::BuildVideoReadTask(name) => write!(f, "Unable to build task for {name}"),
            Error::NoTask => write!(f, "No task to do"),
            Error::SendError(s) => write!(f, "Send Error: {s}"),
            Error::ReceiveError(s) => write!(f, "Receive Error: {s}"),
            Error::IllegalState(s) => write!(f, "Illegal State: {s}"),
            Error::TimerError(s) => write!(f, "Timer Error: {s}"),
        }
    }
}

impl error::Error for Error {}


