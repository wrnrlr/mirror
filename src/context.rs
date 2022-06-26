use crate::Color;

pub struct Window {
  pub event_loop: winit::event_loop::EventLoop<()>,
  pub window: winit::window::Window,
}

impl Window {
  pub fn new()->Window {
    let event_loop = winit::event_loop::EventLoop::new();
    let raw = winit::window::WindowBuilder::new().with_title("Mirror").build(&event_loop).unwrap();

    #[cfg(target_arch = "wasm32")] {
      // Winit prevents sizing with CSS, so we have to set  the size manually when on web.
      use winit::dpi::PhysicalSize;
      raw.set_inner_size(PhysicalSize::new(450, 400));

      use winit::platform::web::WindowExtWebSys;
      web_sys::window()
        .and_then(|win| win.document())
        .and_then(|doc| {
          let dst = doc.get_element_by_id("wasm-example")?;
          let canvas = web_sys::Element::from(raw.canvas());
          dst.append_child(&canvas).ok()?;
          Some(())
        }).expect("Couldn't append canvas to document body.");
    }

    Window{event_loop, window: raw }
  }
}

struct SurfaceContext {
  raw: wgpu::Surface,
  config: wgpu::SurfaceConfiguration,
}

pub struct Target {
  pub view: wgpu::TextureView,
  pub format: wgpu::TextureFormat,
  pub size: wgpu::Extent3d,
}

impl Target {
  pub fn aspect(&self) -> f32 {
    self.size.width as f32 / self.size.height as f32
  }
}

/// Parameters of a texture target that affect its pipeline compatibility.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TargetInfo {
  pub format: wgpu::TextureFormat,
  pub sample_count: u32,
  pub aspect_ratio: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TargetRef(u8);

pub struct Cx {
  #[allow(unused)]
  // window: Window,
  instance: wgpu::Instance,
  surface: Option<SurfaceContext>,
  device: wgpu::Device,
  queue: wgpu::Queue,
  targets: Vec<Target>,
  render_pipeline: wgpu::RenderPipeline,
  // images: Vec<Image>,
  // meshes: Vec<Mesh>,
}

impl Cx {
  pub async fn new(window:&Window) -> Self {
    let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
    let size = window.window.inner_size();
    let mut surface = SurfaceContext {
      raw: unsafe { instance.create_surface(&window.window) },
      config: wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        // using an erroneous format, it is changed before used
        format: wgpu::TextureFormat::Depth24Plus,
        width: size.width, height: size.height,
        present_mode: wgpu::PresentMode::Mailbox,
      },
    };

    let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
        compatible_surface: Some(&surface.raw),
        ..Default::default()}).await.unwrap();

    let (device, queue) = adapter
      .request_device(&wgpu::DeviceDescriptor::default(), None)
      .await.unwrap();

    let format = surface.raw.get_preferred_format(&adapter).unwrap();
    surface.config.format = format;
    surface.raw.configure(&device, &surface.config);

    let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
      label: Some("Shader"),
      source: wgpu::ShaderSource::Wgsl(include_str!("pass/phong.wgsl").into()),
    });

    let render_pipeline_layout =
      device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[],
        push_constant_ranges: &[],
      });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some("Render Pipeline"),
      layout: Some(&render_pipeline_layout),
      vertex: wgpu::VertexState {
        module: &shader,
        entry_point: "vs_main",
        buffers: &[],
      },
      fragment: Some(wgpu::FragmentState {
        module: &shader,
        entry_point: "fs_main",
        targets: &[wgpu::ColorTargetState {
          format: surface.config.format,
          blend: Some(wgpu::BlendState {
            color: wgpu::BlendComponent::REPLACE,
            alpha: wgpu::BlendComponent::REPLACE,
          }),
          write_mask: wgpu::ColorWrites::ALL,
        }],
      }),
      primitive: wgpu::PrimitiveState {
        topology: wgpu::PrimitiveTopology::TriangleList,
        strip_index_format: None,
        front_face: wgpu::FrontFace::Ccw,
        cull_mode: Some(wgpu::Face::Back),
        // Setting this to anything other than Fill requires Features::POLYGON_MODE_LINE
        // or Features::POLYGON_MODE_POINT
        polygon_mode: wgpu::PolygonMode::Fill,
        // Requires Features::DEPTH_CLIP_CONTROL
        unclipped_depth: false,
        // Requires Features::CONSERVATIVE_RASTERIZATION
        conservative: false,
      },
      depth_stencil: None,
      multisample: wgpu::MultisampleState {
        count: 1,
        mask: !0,
        alpha_to_coverage_enabled: false,
      },
      // If the pipeline will be used with a multiview render pass, this
      // indicates how many array layers the attachments will have.
      multiview: None,
    });

    Self {
      // window,
      instance,
      surface: Some(surface),
      device,
      queue,
      targets: Vec::new(),
      render_pipeline
    }
  }

  pub fn resize(&mut self, width: u32, height: u32) {
    let surface = match self.surface {
      Some(ref mut suf) => suf,
      None => return,
    };
    if (surface.config.width, surface.config.height) == (width, height) {
      return;
    }
    surface.config.width = width;
    surface.config.height = height;
    surface.raw.configure(&self.device, &surface.config);
  }

  // pub fn present<P: Pass>(&mut self, pass: &mut P, scene: &Scene, camera: &Camera) {
  //   let surface = self.surface.as_mut().expect("No screen is configured!");
  //   let frame = surface.raw.get_current_texture().unwrap();
  //   let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
  //
  //   let tr = TargetRef(self.targets.len() as _);
  //   self.targets.push(Target {
  //     view,
  //     format: surface.config.format,
  //     size: wgpu::Extent3d {
  //       width: surface.config.width,
  //       height: surface.config.height,
  //       depth_or_array_layers: 1,
  //     },
  //   });
  //
  //   pass.draw(&[tr], scene, camera, self);
  //
  //   self.targets.pop();
  //   frame.present();
  // }
  //
  // pub fn surface_info(&self) -> Option<TargetInfo> {
  //   self.surface.as_ref().map(|s| TargetInfo {
  //     format: s.config.format,
  //     sample_count: 1,
  //     aspect_ratio: s.config.width as f32 / s.config.height as f32,
  //   })
  // }

  pub fn render(&mut self) {
    // self.queue.write_buffer(&self.uniforms_buffer, 0, self.uniforms.as_bytes());
    let surface = self.surface.as_mut().expect("No screen is configured!");
    let frame = surface.raw.get_current_texture().unwrap();
    let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{label: Some("Render Encoder")});
    {
      let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Render Pass"),
        color_attachments: &[wgpu::RenderPassColorAttachment{view: &view, resolve_target: None, ops: wgpu::Operations{load: wgpu::LoadOp::Clear(wgpu::Color{r:0.1,g:0.2,b:0.3,a:1.0}),store: true}}],
        depth_stencil_attachment: None});
      render_pass.set_pipeline(&self.render_pipeline);
      // render_pass.set_bind_group(0, &self.uniforms_bind_group, &[]);
      render_pass.draw(0..3, 0..1);
    }
    self.queue.submit(std::iter::once(encoder.finish()));
    frame.present();
  }
}

// pub trait Pass {
//   fn draw(&mut self, targets: &[TargetRef], scene: &Scene, camera: &Camera, context: &Context);
// }

#[derive(Clone, Debug)]
pub struct Camera {
  // pub projection: Projection,
  /// Specify the depth range as seen by the camera.
  /// `depth.start` maps to 0.0, and `depth.end` maps to 1.0.
  /// Vertical field of view, in degrees.
  /// Note: the horizontal FOV is computed based on the aspect.
  fov_y: f32,
  pub depth: std::ops::Range<f32>,
  // pub node: super::NodeRef, // TODO
  pub background: Color,
}

impl Default for Camera {
  fn default() -> Self {
    Self {
      // Vertical field of view, in degrees...
      fov_y: 160.0,
      depth: 0.0..1.0,
      // node: super::NodeRef::default(),
      background: Color::BLACK,
    }
  }
}

impl Camera {
  pub fn projection_matrix(&self, aspect: f32) -> glam::Mat4 {
    let fov = self.fov_y.to_radians();
    let matrix = if self.depth.end == f32::INFINITY {
          assert!(self.depth.start.is_finite());
          glam::Mat4::perspective_infinite_rh(fov, aspect, self.depth.start)
        } else if self.depth.start == f32::INFINITY {
          glam::Mat4::perspective_infinite_reverse_rh(fov, aspect, self.depth.end)
        } else {
          glam::Mat4::perspective_rh(fov, aspect, self.depth.start, self.depth.end)
        };
    matrix
  }
}