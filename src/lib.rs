use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
    window::{Window, WindowBuilder},
};

pub async fn run() {
    env_logger::init();

    //BIG TODO: Custom error handling
    let event_loop = EventLoop::new().unwrap();

    let window = WindowBuilder::new().build(&event_loop).unwrap();

    event_loop.run(move |event, window| handle_event(event, window));
}

fn handle_event(event: Event<()>, window: &EventLoopWindowTarget<()>) {
    match event {
        Event::WindowEvent {
            event: ref window_event,
            window_id,
        } => match window_event {
            WindowEvent::CloseRequested => {
                window.exit();
            }
            _ => {}
        },
        _ => {}
    };
}
