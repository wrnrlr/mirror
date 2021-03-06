use g3::{Point,Plane,point,E2};
use glam::{Vec3};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
  position:[f32;3],
}

impl Vertex {
  pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
    wgpu::VertexBufferLayout {
      array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
      step_mode: wgpu::VertexStepMode::Vertex,
      attributes: &[
        wgpu::VertexAttribute {offset: 0, shader_location: 0, format: wgpu::VertexFormat::Float32x3, },
        wgpu::VertexAttribute {
          offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
          shader_location: 1, format: wgpu::VertexFormat::Float32x3,
        },
      ],
    }
  }
}

pub fn create_plane_mesh(p:Plane)-> Mesh {
  let p = p.normalized(); let m = (p*E2).sqrt();
  let a = m(point(-1.0,0.0,-1.0)); let b = m(point(-1.0,0.0,1.0));
  let c = m(point(1.0,0.0,1.0)); let d = m(point(1.0,0.0,-1.0));
  println!("{:?}", a);
  println!("{:?}", b);
  println!("{:?}", c);
  println!("{:?}", d);
  let vertices:Vec<[f32;3]> = vec!(a.into(), b.into(), c.into(), d.into());
  let indices = vec!(0u32, 2, 1, 0, 3, 2, 2, 3, 0, 1, 2, 0);
  // normals??
  Mesh{vertices,indices}
}

pub fn demo_mesh()->Mesh {
  let vertices = vec!([-0.0868241, 0.49240386, 0.0], [-0.49513406, 0.06958647, 0.0], [-0.21918549, -0.44939706, 0.0], [0.35966998, -0.3473291, 0.0], [0.44147372, 0.2347359, 0.0]);
  let indices = vec!(0, 1, 4, 1, 2, 4, 2, 3, 4);
  Mesh{vertices,indices}
}

#[derive(Debug)]
pub struct Mesh {
  pub vertices:Vec<[f32;3]>,
  pub indices:Vec<u32>
}

impl Mesh {
  pub fn new(points:Vec<Point>)->MeshBuilder {
    MeshBuilder{positions:vec![]}
  }
}

#[derive(Debug, Default)]
pub struct MeshBuilder {
  positions:Vec<[f32;3]>
}

impl MeshBuilder {
  // pub fn build(self)->Mesh {
  //   Mesh{vertices:self.positions,indices}
  // }
}

pub struct Geometry {

}
