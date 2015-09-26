#[macro_use]
extern crate mioqu;
extern crate mio;
use std::thread::sleep_ms;
use mioqu::{Queue, Handler, EventLoop, Token, Callback, ResponseResult};

#[derive(Debug)]
enum Job {
    Counter(usize, u64),
}

#[derive(Debug)]
enum Task {
    Start,
    Count
}

#[derive(Debug)]
enum Response {
    Numeric(usize)
}

impl Response {
    fn as_num(&self) -> ResponseResult<usize> {
        match *self {
            Response::Numeric(val) => Ok(val),
            //_ => Err(ResponseError("invalid type"))
        }
    }
}

#[derive(Debug)]
struct All;

#[derive(Debug)]
struct MyHandler;

impl mioqu::Handler for MyHandler  {
    type Processor = Job;
    type Message   = Task;
    type Response  = Response;
    type Timeout   = All;

    fn process(&mut self, event_loop: &mut EventLoop<Self>, token: Token, job: &mut Job, task: Task, callback: Callback<Response>) {
        match *job {
            Job::Counter(counter, period) => {
                match task {
                    Task::Start => {
                        event_loop.timeout_ms(mioqu::Timeout(token, All), period).unwrap();
                    },
                    Task::Count => {
                        callback.reply(Response::Numeric(counter));
                    }
                }
            }
        }
    }

    fn timeout(&mut self, event_loop: &mut EventLoop<Self>, token: Token, job: &mut Job, _timeout: Self::Timeout) {
        match *job {
            Job::Counter(ref mut counter, period) => {
                *counter += 1;
                event_loop.timeout_ms(mioqu::Timeout(token, All), period).unwrap();
            }
        }
    }

    fn tick(&mut self, _event_loop: &mut EventLoop<Self>) {
    }
}


fn main() {
    let cfg = mio::EventLoopConfig {
        io_poll_timeout_ms: 1_000,
        notify_capacity: 4_096,
        messages_per_tick: 256,
        timer_tick_ms: 10,
        timer_wheel_size: 1_024,
        timer_capacity: 65_536,
    };
    let event_loop = EventLoop::configured(cfg).unwrap();
    // run queue
    let mut binding = Queue::run(event_loop, MyHandler).ok().expect("Queue was broken");
    // register job Counter(start, period in ms)
    let token = binding.register(Job::Counter(0, 190)).unwrap();
    // start job
    binding.send(token, Task::Start, Callback::None);
    // wait a litte time
    sleep_ms(2000);
    // get counter value
    let num = queue_wait!(binding, token, Task::Count).as_num().unwrap();
    assert!(num >= 9);
    println!("Counter is {:}", num);
}
