#![feature(unboxed_closures)]
#![feature(fn_traits)]
#![feature(portable_simd)]
#![feature(adt_const_params)]
#![allow(incomplete_features)]

#[cfg(target_arch="wasm32")]
use wasm_bindgen::prelude::*;

mod pass;
mod context;
mod color;
mod mesh;

pub use color::Color;
pub use context::{Window, Cx};

use winit::{event::{Event,WindowEvent},event_loop::ControlFlow};

fn logging() {
  cfg_if::cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
      std::panic::set_hook(Box::new(console_error_panic_hook::hook));
      console_log::init_with_level(log::Level::Warn).expect("Could't initialize logger");
    } else {
      env_logger::init();
    }
  }
}

struct App {
  // pub event_loop: winit::event_loop::EventLoop<()>,
  pub window:Window,
  pub cx: Cx
}

impl App {
  async fn new()->App {
    logging();
    let window = Window::new();
    let cx = Cx::new(&window).await;
    App{window,cx}
  }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
  let App{window,mut cx} = App::new().await;
  window.event_loop.run(move |e, _, control_flow| {
    match e {
      Event::RedrawRequested(window_id) if (window_id==window.window.id()) => { cx.render() }
      Event::MainEventsCleared => { window.window.request_redraw() }
      Event::WindowEvent{ref event,window_id} if (window_id==window.window.id()) => {
        match event {
          WindowEvent::CloseRequested => {*control_flow = ControlFlow::Exit},
          WindowEvent::Resized(physical_size) => { cx.resize(physical_size.width, physical_size.height); }
          WindowEvent::ScaleFactorChanged{new_inner_size, ..} => { cx.resize(new_inner_size.width, new_inner_size.height); }
          _ => {}
        }
      }
      _ => {}
    }
  });
}
