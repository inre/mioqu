use queue::{Callback, EventLoop};
use mio::{Token, EventSet};

#[allow(unused_variables)]
pub trait Handler: Send + Sized + 'static {
    type Processor: Send;
    type Message:   Send;
    type Response:  Send;
    type Timeout:   Send;

    fn ready(&mut self, event_loop: &mut EventLoop<Self>, token: Token, processor: &mut Self::Processor, events: EventSet) {
    }

    /// Invoked when a message has been received via the event loop's channel.
    fn process(&mut self, event_loop: &mut EventLoop<Self>, token: Token, processor: &mut Self::Processor, msg: Self::Message, callback: Callback<Self::Response>) {
    }

    /// Invoked when a timeout has completed.
    fn timeout(&mut self, event_loop: &mut EventLoop<Self>, token: Token, processor: &mut Self::Processor, timeout: Self::Timeout) {
    }

    /// Invoked when `EventLoop` has been interrupted by a signal interrupt.
    /*fn interrupted(&mut self, event_loop: &mut EventLoop<Self>) {
    }*/

    fn tick(&mut self, event_loop: &mut EventLoop<Self>) {
    }
}
