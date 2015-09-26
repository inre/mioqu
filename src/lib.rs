extern crate mio;

mod result;
mod fragments;
mod handler;
mod queue;

pub use mio::{
    Token
};

pub use queue::{
    Queue,
    EventLoop,
    Binding,
    Callback,
    Timeout
};

pub use handler::{
    Handler
};

pub use result::{
    Result,
    Error,
    ResponseError,
    ResponseResult
};

pub use fragments::{
    Index,
    Fragments
};

impl fragments::Index for mio::Token {
    fn from_usize(inner: usize) -> mio::Token {
        mio::Token(inner)
    }

    fn as_usize(&self) -> usize {
        let mio::Token(inner) = *self;
        inner
    }
}
