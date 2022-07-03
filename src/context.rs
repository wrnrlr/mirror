use wgpu::util::DeviceExt;
use crate::Color;
use crate::mesh::{create_plane_mesh, Vertex};

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

impl SurfaceContext {
  pub fn aspect_ratio(&self)->f32 {
    self.config.width as f32 / self.config.height as f32
  }
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

  camera: Camera,
  camera_uniform: CameraUniform,
  camera_buffer: wgpu::Buffer,
  camera_bind_group: wgpu::BindGroup,

  vertex_buffer: wgpu::Buffer,
  index_buffer: wgpu::Buffer,
  num_indices: u32,
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

    // WebGL doesn't support all of wgpu's features, so if we're building for the web we'll have to disable some.
    let (device, queue) = adapter
      .request_device(&wgpu::DeviceDescriptor {
        label: None, features: wgpu::Features::empty(),
        limits: if cfg!(target_arch = "wasm32") {wgpu::Limits::downlevel_webgl2_defaults()}else{wgpu::Limits::default()}}, None).await.unwrap();

    let format = surface.raw.get_preferred_format(&adapter).unwrap();
    surface.config.format = format;
    surface.raw.configure(&device, &surface.config);

    println!("{}, {}", surface.config.width, surface.config.height);

    let shader_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
      label: Some("Phong Shader"),
      source: wgpu::ShaderSource::Wgsl(include_str!("pass/phong.wgsl").into()),
    });

    let aspect_ratio = size.width as f32 / size.height as f32;
    let camera = Camera::default();
    let mut camera_uniform = CameraUniform::new();
    camera_uniform.update_view_proj(&camera, aspect_ratio);
    let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("Camera Buffer"),
      contents: bytemuck::cast_slice(&[camera_uniform]),
      usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST});
    let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      entries: &[
        wgpu::BindGroupLayoutEntry {
          binding: 0,
          visibility: wgpu::ShaderStages::VERTEX,
          ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
          },
          count: None,
        }
      ],
      label: Some("camera_bind_group_layout"),
    });
    let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
      layout: &camera_bind_group_layout,
      entries: &[wgpu::BindGroupEntry {binding: 0, resource: camera_buffer.as_entire_binding()}],
      label: Some("camera_bind_group"),
    });

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("Render Pipeline Layout"),
      bind_group_layouts: &[&camera_bind_group_layout],
      push_constant_ranges: &[]});

    let target_info_format = &[wgpu::ColorTargetState {
      format: surface.config.format,
      blend: Some(wgpu::BlendState {
        color: wgpu::BlendComponent::REPLACE,
        alpha: wgpu::BlendComponent::REPLACE,
      }),
      write_mask: wgpu::ColorWrites::ALL}];
    let primitive = wgpu::PrimitiveState{cull_mode:Some(wgpu::Face::Back),..Default::default()};
    let multisample = wgpu::MultisampleState{count:1, ..Default::default()};
    let depth_stencil = None;

    // render pipeline for phong...
    let vertex_buffers = &[Vertex::desc()];
    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some("phong"),
      layout: Some(&render_pipeline_layout),
      vertex: wgpu::VertexState {buffers: vertex_buffers, module: &shader_module, entry_point: "vs_main"},
      primitive,
      depth_stencil: depth_stencil.clone(),
      multisample,
      fragment: Some(wgpu::FragmentState {
        targets: target_info_format,
        module: &shader_module, entry_point: "fs_main",
      }),
      multiview: None,
    });

    let mesh = &create_plane_mesh(g3::E3);
    println!("{:?}", mesh);

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("Vertex Buffer"),
      contents: bytemuck::cast_slice(&mesh.vertices),
      usage: wgpu::BufferUsages::VERTEX});
    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("Index Buffer"),
      contents: bytemuck::cast_slice(&mesh.indices),
      usage: wgpu::BufferUsages::INDEX});
    let num_indices = mesh.indices.len() as u32;
    println!("indices: {:?}",num_indices);

    Self {
      // window,
      instance,
      surface: Some(surface),
      device,
      queue,
      targets: Vec::new(),
      render_pipeline,

      camera,
      camera_uniform,
      camera_buffer,
      camera_bind_group,

      vertex_buffer,
      index_buffer,
      num_indices
    }
  }

  pub fn add_plane(&self) {
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


  //     fn inverse(&self) -> Self {
  //         let scale = 1.0 / self.scale;
  //         let orientation = self.orientation.inverse();
  //         let position = -scale * (orientation * self.position);
  //         Self {
  //             position,
  //             scale,
  //             orientation,
  //         }
  //     }

  pub fn render(&mut self) {
    // self.queue.write_buffer(&self.uniforms_buffer, 0, self.uniforms.as_bytes());
    let surface = self.surface.as_mut().expect("No screen is configured!");
    let frame = surface.raw.get_current_texture().unwrap();
    let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{label: Some("Render Encoder")});
    {
      let aspect = surface.aspect_ratio();
      println!("{}", aspect);
      let m_proj = self.camera.projection_matrix(aspect);
      let m_view = self.camera.view_matrix();
      // let m_view_inv = nodes[camera.node].inverse_matrix();
      // let m_final = m_proj * glam::Mat4::from(m_view_inv);
      // let globals = Globals { view_proj: m_final.to_cols_array_2d() };
      let m = m_proj * m_view;
      let globals = CameraUniform{view_proj: m.to_cols_array_2d()};
      self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::bytes_of(&globals));
    }
    {
      let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Render Pass"),
        color_attachments: &[wgpu::RenderPassColorAttachment{view: &view, resolve_target: None, ops: wgpu::Operations{load: wgpu::LoadOp::Clear(wgpu::Color{r:0.1,g:0.2,b:0.3,a:1.0}),store: true}}],
        depth_stencil_attachment: None});
      render_pass.set_pipeline(&self.render_pipeline);
      render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
      render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
      render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
      render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
    }
    self.queue.submit(Some(encoder.finish()));
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
  eye:[f32;3],
  target:[f32;3],
  up:[f32;3],
  fov_y: f32,
  pub depth: std::ops::Range<f32>,
  // pub node: super::NodeRef, // TODO
  pub background: Color,
}

impl Default for Camera {
  fn default() -> Self {
    Self {
      eye: [0.0, 1.0, 2.0].into(),
      target: [0.0, 0.0, 0.0].into(),
      up: [0.0, 1.0, 0.0].into(),
      // Vertical field of view, in degrees...
      fov_y: 160.0,
      depth: -0.1..100.0, // 0.0..1.0
      // node: super::NodeRef::default(),
      background: Color::BLACK,
    }
  }
}

impl Camera {
  pub fn view_matrix(&self) -> glam::Mat4 {
    glam::Mat4::look_at_rh(self.eye.into(), self.target.into(), self.up.into())
  }
  pub fn projection_matrix(&self, aspect: f32) -> glam::Mat4 {
    let fov = self.fov_y.to_radians();
    // let matrix = if self.depth.end == f32::INFINITY {
    //       assert!(self.depth.start.is_finite());
    //       glam::Mat4::perspective_infinite_rh(fov, aspect, self.depth.start)
    //     } else if self.depth.start == f32::INFINITY {
    //       glam::Mat4::perspective_infinite_reverse_rh(fov, aspect, self.depth.end)
    //     } else {
    //       glam::Mat4::perspective_rh(fov, aspect, self.depth.start, self.depth.end)
    //     };
    // matrix
    glam::Mat4::perspective_rh(fov, aspect, -0.1, 1.0)
  }
}

#[repr(C)] #[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
  view_proj:[[f32;4];4]
}

impl CameraUniform {
  fn new() -> Self {
    Self{view_proj: glam::Mat4::IDENTITY.to_cols_array_2d()}
  }
  fn update_view_proj(&mut self, camera: &Camera, aspect:f32) {
    self.view_proj = camera.projection_matrix(aspect).to_cols_array_2d();
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test] fn plane_mesh() {
    let p = create_plane_mesh(g3::E3);
    println!("{:?}", p);
  }

  #[test] fn camera() {
    let mat = glam::Mat4::perspective_infinite_rh(160f32.to_radians(), 4.0/3.0, 0.0);
    println!("{}",mat);
  }
}
