use std::cell::RefCell;
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::result;
use mio::{self, Token, EventSet};
use fragments::Fragments;
use {Handler, Result, Error};

pub type EventLoop<H: Handler> = mio::EventLoop<Queue<H>>;

pub struct Queue<H: Handler> {
    handler: RefCell<H>,
    processors: Fragments<RefCell<H::Processor>, Token>,
}

impl<H: Handler> mio::Handler for Queue<H> {
    type Timeout = Timeout<H::Timeout>;
    type Message = Message<H::Processor, H::Message, H::Response>;

    fn ready(&mut self, event_loop: &mut EventLoop<H>, token: Token, events: EventSet) {
        let processor = self.processors.elem_mut(token).as_mut().unwrap();
        let mut hnd = self.handler.borrow_mut();
        hnd.ready(event_loop, token, &mut *processor.borrow_mut(), events);
    }

    fn notify(&mut self, event_loop: &mut EventLoop<H>, message: Self::Message) {
        match message {
            Message::ControlMessage(cm) => {
                match cm {
                    ControlMessage::Initialize(sender) => {
                        sender.send(true).unwrap();
                    },

                    // Activity

                    ControlMessage::RegisterProcessor(processor, sender) => {
                        let token = self.register(processor);
                        sender.send(token).unwrap();
                    },
                    ControlMessage::UnregisterProcessor(token) => {
                        self.unregister(token);
                    },
                }
            },
            Message::UserMessage(token, um, callback) => {
                let processor = self.processors.elem_mut(token).as_mut().unwrap();
                let mut hnd = self.handler.borrow_mut();
                hnd.process(event_loop, token, &mut *processor.borrow_mut(), um, callback);
            }
        }
    }

    fn timeout(&mut self, event_loop: &mut EventLoop<H>, timeout: Self::Timeout) {
        let token = timeout.0;
        let processor = self.processors.elem_mut(token).as_mut().unwrap();
        let mut hnd = self.handler.borrow_mut();
        hnd.timeout(event_loop, token, &mut *processor.borrow_mut(), timeout.1);
    }
/*
    fn interrupted(&mut self, event_loop: &mut EventLoop<H>) {
        let mut hnd = self.handler.borrow_mut();
        hnd.interrupted(event_loop);
    }
*/
    fn tick(&mut self, event_loop: &mut EventLoop<H>) {
        let mut hnd = self.handler.borrow_mut();
        hnd.tick(event_loop);
    }
}

impl<H: Handler> Queue<H> {

    pub fn run(event_loop: EventLoop<H>, handler: H) -> result::Result<Binding<H>, Error<H>> {
        let sender = event_loop.channel();
        let (tx, rx) = channel();

        thread::spawn(move || {
            // Procs don't allow capturing mutable variables #10617
            let mut event_loop = event_loop;
            let mut queue = Queue { handler: RefCell::new(handler), processors: Fragments::new() };

            // initial event
            let msg = Message::ControlMessage(ControlMessage::Initialize(tx));
            event_loop.channel().send(msg).unwrap();

            // run reactor
            event_loop.run(&mut queue).unwrap();
        });

        // wait until reactor started
        if rx.recv().is_err() {
            return Err(Error::QueueOutOfService);
        }

        Ok(Binding { sender: sender })
    }

    pub fn register(&mut self, processor: H::Processor) -> Token {
        self.processors.add(RefCell::new(processor))
    }

    pub fn unregister(&mut self, token: Token) {
        self.processors.delete(token);
    }
}

pub struct Binding<H: Handler> {
    // Send notifies to Queue
    sender: mio::Sender<Message<H::Processor, H::Message, H::Response>>
}

impl<H: Handler> Clone for Binding<H> {
    fn clone(&self) -> Self {
        Binding { sender: self.sender.clone() }
    }
}

impl<H: Handler> Binding<H> {
    pub fn send(&mut self, token: Token, user_message: H::Message, callback: Callback<H::Response>) {
        self.sender.send(Message::UserMessage(token, user_message, callback)).unwrap();
    }

    pub fn register(&mut self, processor: H::Processor) -> Result<Token, H> {
        let (tx, rx) = channel();
        let msg = Message::ControlMessage(ControlMessage::RegisterProcessor(processor, tx));
        self.sender.send(msg).unwrap();
        Ok(try!(rx.recv()))
    }

    pub fn unregister(&mut self, token: Token) {
        let msg = Message::ControlMessage(ControlMessage::UnregisterProcessor(token));
        let _ = self.sender.send(msg);
    }
}

pub struct Timeout<U>(pub Token, pub U);

pub enum Message<P, U, R: Send> {
    UserMessage(Token, U, Callback<R>),
    ControlMessage(ControlMessage<P>),
}

enum ControlMessage<P> {
    Initialize(Sender<bool>),
    RegisterProcessor(P, Sender<Token>),
    UnregisterProcessor(Token),
}

pub enum Callback<T: Send> {
    Notify(mio::Sender<T>),
    Channel(Sender<T>),
    None,
}

impl<T> Callback<T> where T: Send {
    pub fn reply(&self, response: T) {
        match *self {
            Callback::Notify(ref sender) => sender.send(response).unwrap(),
            Callback::Channel(ref tx)    => tx.send(response).unwrap(),
            Callback::None               => (),
        }
    }
}

#[macro_export]
macro_rules! queue_wait {
    ($binding: ident, $token: expr, $um: expr) => {{
        use std::sync::mpsc::channel;
        let (tx, rx) = channel();
        $binding.send($token, $um, Callback::Channel(tx));
        rx.recv().unwrap()
    }};
}

#[macro_use]
#[cfg(test)]
mod test {
    use mio::Token;
    use std::sync::mpsc::channel;
    use super::{Queue, EventLoop, Callback};
    use super::super::handler::Handler;
    use super::super::result::{ResponseResult, ResponseError};

    enum Storage {
        Numeric(usize),
        Word(&'static str)
    }

    enum Query {
        Save(CustomType),
        Load,
    }

    enum CustomType {
        OK,
        Num(usize),
        Str(&'static str),
    }

    impl CustomType {
        fn as_num(&self) -> ResponseResult<usize> {
            match *self {
                CustomType::Num(val) => Ok(val),
                _ => Err(ResponseError("incorrect type"))
            }
        }

        fn as_str(&self) -> ResponseResult<&'static str> {
            match *self {
                CustomType::Str(val) => Ok(val),
                _ => Err(ResponseError("incorrect type"))
            }
        }

        fn ok(&self) -> ResponseResult<()> {
            match *self {
                CustomType::OK => Ok(()),
                _ => Err(ResponseError("incorrect type"))
            }
        }
    }

    struct StorageHandler;

    impl Handler for StorageHandler {
        type Processor = Storage;
        type Message   = Query;
        type Response  = CustomType;
        type Timeout   = Token;

        fn process(&mut self, _event_loop: &mut EventLoop<Self>, _token: Token,
            storage: &mut Storage, query: Query, callback: Callback<CustomType>) {

            match *storage {
                Storage::Numeric(ref mut num) => {
                    match query {
                        Query::Save(val) => {
                            *num = val.as_num().unwrap();
                            callback.reply(CustomType::OK);
                        },
                        Query::Load => {
                            callback.reply(CustomType::Num(*num));
                        }
                    }
                },

                Storage::Word(ref mut word) => {
                    match query {
                        Query::Save(val) => {
                            *word = val.as_str().unwrap();
                            callback.reply(CustomType::OK);
                        },
                        Query::Load => {
                            callback.reply(CustomType::Str(*word));
                        }
                    }
                },
            }
        }
    }

    #[test]
    fn test_send_message_and_wait_response() {
        let event_loop = EventLoop::new().unwrap();
        let mut binding = Queue::run(event_loop, StorageHandler).ok().expect("Queue was broken");

        let token = binding.register(Storage::Numeric(13)).ok().expect("Storage was not available");

        // Load value
        let (tx, rx) = channel();
        binding.send(token, Query::Load, Callback::Channel(tx));
        let value = rx.recv().unwrap().as_num().unwrap();
        assert_eq!(value, 13);

        // Save new value
        let (tx, rx) = channel();
        binding.send(token, Query::Save(CustomType::Num(18)), Callback::Channel(tx));
        let _ = rx.recv().unwrap().ok().unwrap();

        // Load value
        let (tx, rx) = channel();
        binding.send(token, Query::Load, Callback::Channel(tx));
        let value = rx.recv().unwrap().as_num().unwrap();
        assert_eq!(value, 18);
    }

    #[test]
    fn test_macro_act_wait() {
        let event_loop = EventLoop::new().unwrap();
        let mut binding = Queue::run(event_loop, StorageHandler).ok().expect("Queue was broken");

        let token = binding.register(Storage::Word("foo")).ok().expect("Storage was not available");

        let str = queue_wait!(binding, token, Query::Load).as_str().unwrap();
        assert_eq!(str, "foo");
    }
}
