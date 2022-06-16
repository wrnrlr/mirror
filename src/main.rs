use winit::{event::{Event,WindowEvent},event_loop::ControlFlow};
use mirror::{Context,Window};

fn main() {
  // env_logger::init();
  // let event_loop:EventLoop<ReloadEvent> = EventLoop::with_user_event();
  // let proxy:EventLoopProxy<ReloadEvent> = event_loop.create_proxy();

  let mut window = Window::new();
  let mut cx = pollster::block_on(Context::new(&window));

  let Window{event_loop,raw} = window;
  event_loop.run(move |event, _, control_flow| {
    match event {
      Event::RedrawRequested(_) => { cx.render() }
      Event::MainEventsCleared => { raw.request_redraw() }
      Event::WindowEvent {event:WindowEvent::CloseRequested,..} => { *control_flow = ControlFlow::Exit }
      _ => {}
    }
  });
}
