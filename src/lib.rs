use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

pub fn run() {
    env_logger::init();

    //BIG TODO: Custom error handling
    let event_loop = EventLoop::new().unwrap();

    let window = WindowBuilder::new().build(&event_loop).unwrap();

    event_loop.run(move |event, _| handle_event(event));

    //todo
    //  get instance
    //  get surface
    //  display in window
}

fn handle_event(event: Event<()>) {
    print!("Got an event\n")
}
