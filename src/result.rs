use std::io::Error as IoError;
use std::sync::mpsc::{SendError, RecvError};
use std::result;
use mio::NotifyError;
use queue::Message;
use {Handler};

#[derive(Debug)]
pub enum Error<H: Handler> {
    QueueOutOfService,
    Io(IoError),
    NotifyError(NotifyError<Message<H::Processor, H::Message, H::Response>>),
    SendError(SendError<H::Response>),
    RecvError(RecvError),
}

pub type Result<T, H: Handler> = result::Result<T, Error<H>>;

impl<H: Handler> From<IoError> for Error<H> {
    fn from(err: IoError) -> Error<H> {
        Error::Io(err)
    }
}

impl<H: Handler> From<NotifyError<Message<H::Processor, H::Message, H::Response>>> for Error<H> {
    fn from(err: NotifyError<Message<H::Processor, H::Message, H::Response>>) -> Error<H> {
        Error::NotifyError(err)
    }
}

impl<H: Handler> From<SendError<H::Response>> for Error<H> {
    fn from(err: SendError<H::Response>) -> Error<H> {
        Error::SendError(err)
    }
}

impl<H: Handler> From<RecvError> for Error<H> {
    fn from(err: RecvError) -> Error<H> {
        Error::RecvError(err)
    }
}

#[derive(Debug)]
pub struct ResponseError(pub &'static str);

pub type ResponseResult<T> = result::Result<T, ResponseError>;
