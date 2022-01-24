#![feature(unboxed_closures)]
#![feature(fn_traits)]
#![feature(portable_simd)]
#![feature(adt_const_params)]
#![allow(incomplete_features)]

mod shape;
mod color;

pub use color::Color;

use g3::*;
use std::sync::Arc;
use glam::{Mat4, Vec3, Vec4};
use rend3::types::{CameraProjection, DirectionalLightHandle, Handedness, ObjectHandle, ObjectMeshKind};
pub use rend3_framework::{App,start};
use rend3_routine::pbr::{AlbedoComponent, PbrMaterial};


const SAMPLE_COUNT: rend3::types::SampleCount = rend3::types::SampleCount::One;

pub struct Scene {}


#[derive(Default)]
pub struct Mirror {
  object_handle: Option<ObjectHandle>,
  directional_light_handle: Option<DirectionalLightHandle>,
  world: hecs::World,
  points: Vec<(Point, Color)>,
  lines: Vec<(Line, Color)>,
  planes: Vec<(Plane, Color)>,
  faces: Vec<(Vec<[Point; 3]>, Color)>,
  objects: Vec<ObjectHandle>
}

impl Mirror {
  pub fn vertex(&mut self, p:Point, c:Color) {
    self.points.push((p, c));
  }

  pub fn face(&mut self, face:[Point; 3], c:Color) {
    self.faces.push((vec![face], c));
  }
}

const POINT_RADIUS:f32 = 0.1;

impl App for Mirror {
  const HANDEDNESS: Handedness = Handedness::Left;

  fn sample_count(&self) -> rend3::types::SampleCount {
    SAMPLE_COUNT
  }

  fn setup(
    &mut self,
    _window: &winit::window::Window,
    renderer: &Arc<rend3::Renderer>,
    _routines: &Arc<rend3_framework::DefaultRoutines>,
    _surface_format: rend3::types::TextureFormat,
  ) {
    for p in &self.points {
      let (p,c) = p;
      let sphere_mesh = shape::Shape::sphere(POINT_RADIUS);
      let sphere_mesh_handle = renderer.add_mesh(sphere_mesh);
      let material = PbrMaterial {
        albedo: AlbedoComponent::Value(c.into()),
        ..PbrMaterial::default()
      };
      let material_handle = renderer.add_material(material);
      let transf = Mat4::from_translation([p.x(), p.y(), p.z()].into());
      let object = rend3::types::Object {
        mesh_kind: ObjectMeshKind::Static(sphere_mesh_handle),
        material: material_handle,
        transform: transf,
      };
      self.objects.push(renderer.add_object(object));
    }

    for p in &self.faces {
      let (f,c) = p;
      let sphere_mesh = shape::Shape::surface(f);
      let sphere_mesh_handle = renderer.add_mesh(sphere_mesh);
      let material = PbrMaterial {
        albedo: AlbedoComponent::Value(c.into()),
        ..PbrMaterial::default()
      };
      let material_handle = renderer.add_material(material);
      let transf = Mat4::IDENTITY;
      let object = rend3::types::Object {
        mesh_kind: ObjectMeshKind::Static(sphere_mesh_handle),
        material: material_handle,
        transform: transf,
      };
      self.objects.push(renderer.add_object(object));
    }

    let view_location = Vec3::new(3.0, 3.0, -5.0);
    let view = Mat4::from_euler(glam::EulerRot::XYZ, -0.55, 0.5, 0.0);
    let view = view * Mat4::from_translation(-view_location);

    // Set camera's location
    renderer.set_camera_data(rend3::types::Camera {
      projection: CameraProjection::Perspective { vfov: 60.0, near: 0.1 },
      view,
    });

    // Create a single directional light
    //
    // We need to keep the directional light handle alive.
    self.directional_light_handle = Some(renderer.add_directional_light(rend3::types::DirectionalLight {
      color: Color::WHITE.into(),
      intensity: 10.0,
      // Direction will be normalized
      direction: Vec3::new(-1.0, -4.0, 2.0),
      distance: 400.0,
    }));
  }

  fn handle_event(
    &mut self,
    window: &winit::window::Window,
    renderer: &Arc<rend3::Renderer>,
    routines: &Arc<rend3_framework::DefaultRoutines>,
    base_rendergraph: &rend3_routine::base::BaseRenderGraph,
    surface: Option<&Arc<rend3::types::Surface>>,
    resolution: glam::UVec2,
    event: rend3_framework::Event<'_, ()>,
    control_flow: impl FnOnce(winit::event_loop::ControlFlow),
  ) {
    match event {
      // Close button was clicked, we should close.
      rend3_framework::Event::WindowEvent {
        event: winit::event::WindowEvent::CloseRequested,
        ..
      } => {
        control_flow(winit::event_loop::ControlFlow::Exit);
      }
      rend3_framework::Event::MainEventsCleared => {
        window.request_redraw();
      }
      // Render!
      rend3_framework::Event::RedrawRequested(_) => {
        // Get a frame
        let frame = rend3::util::output::OutputFrame::Surface {
          surface: Arc::clone(surface.unwrap()),
        };
        // Ready up the renderer
        let (cmd_bufs, ready) = renderer.ready();

        // Lock the routines
        let pbr_routine = rend3_framework::lock(&routines.pbr);
        let tonemapping_routine = rend3_framework::lock(&routines.tonemapping);

        // Build a rendergraph
        let mut graph = rend3::graph::RenderGraph::new();

        // Add the default rendergraph without a skybox
        base_rendergraph.add_to_graph(
          &mut graph,
          &ready,
          &pbr_routine,
          None,
          &tonemapping_routine,
          resolution,
          SAMPLE_COUNT,
          Vec4::ZERO,
        );

        // Dispatch a render using the built up rendergraph!
        graph.execute(renderer, frame, cmd_bufs, &ready);
      }
      // Other events we don't care about
      _ => {}
    }
  }
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
