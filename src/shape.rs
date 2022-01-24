use g3::Point;
use rend3::types::Mesh;
use glam::Vec3;

fn vertex(pos: [f32; 3]) -> glam::Vec3 {
  glam::Vec3::from(pos)
}

pub struct Shape;

impl Shape {
  pub fn plane(_size:f32) {}

  pub fn cylinder() {}

  pub fn sphere(radius:f32)->Mesh {
    const F: f32 = 1.618034; // 0.5 * (1.0 + 5f32.sqrt());

    // Base icosahedron positions [[f32; 3]; 12]
    const BASE_POSITIONS:[[f32; 3]; 12] = [
      [-1.0, F, 0.0],
      [1.0, F, 0.0],
      [-1.0, -F, 0.0],
      [1.0, -F, 0.0],
      [0.0, -1.0, F],
      [0.0, 1.0, F],
      [0.0, -1.0, -F],
      [0.0, 1.0, -F],
      [F, 0.0, -1.0],
      [F, 0.0, 1.0],
      [-F, 0.0, -1.0],
      [-F, 0.0, 1.0],
    ];

    // Base icosahedron faces
    const BASE_FACES: [[u32; 3]; 20] = [
      [0, 11, 5],
      [0, 5, 1],
      [0, 1, 7],
      [0, 7, 10],
      [0, 10, 11],
      [11, 10, 2],
      [5, 11, 4],
      [1, 5, 9],
      [7, 1, 8],
      [10, 7, 6],
      [3, 9, 4],
      [3, 4, 2],
      [3, 2, 6],
      [3, 6, 8],
      [3, 8, 9],
      [9, 8, 1],
      [4, 9, 5],
      [2, 4, 11],
      [6, 2, 10],
      [8, 6, 7],
    ];

    let detail = 3;

    let mut lookup = fxhash::FxHashMap::default();
    let mut prev_faces = Vec::new();
    let mut vertices = BASE_POSITIONS
      .iter()
      .map(|p| glam::Vec3::from_slice(p))
      .collect::<Vec<_>>();
    let mut faces = BASE_FACES.to_vec();

    for _ in 1..detail {
      lookup.clear();
      prev_faces.clear();
      prev_faces.append(&mut faces);

      for face in prev_faces.iter() {
        let mut mid = [0u32; 3];
        for (pair, index) in face
          .iter()
          .cloned()
          .zip(face[1..].iter().chain(face.first()).cloned())
          .zip(mid.iter_mut())
        {
          *index = match lookup.get(&pair) {
            Some(i) => *i,
            None => {
              let i = vertices.len() as u32;
              lookup.insert(pair, i);
              lookup.insert((pair.1, pair.0), i);
              let v = 0.5 * (vertices[pair.0 as usize] + vertices[pair.1 as usize]);
              vertices.push(v);
              i
            }
          };
        }

        faces.push([face[0], mid[0], mid[2]]);
        faces.push([face[1], mid[1], mid[0]]);
        faces.push([face[2], mid[2], mid[1]]);
        faces.push([mid[0], mid[1], mid[2]]);
      }
    }

    let indices = faces.into_iter().flat_map(|face| face).collect::<Vec<_>>();
    let mut positions = Vec::with_capacity(vertices.len());
    let mut normals = Vec::with_capacity(vertices.len());

    for v in vertices {
      let n = v.normalize();
      positions.push(n * radius);
      normals.push(n);
      // if let Some(ref mut normals) = normals {
      //   normals.push(crate::Normal(n.into()));
      // }
    }

    rend3::types::MeshBuilder::new(positions, rend3::types::Handedness::Left)
      .with_indices(indices)
      .with_vertex_normals(normals)
      .build()
      .unwrap()
  }

  pub fn cube()->Mesh {
    let vertex_positions = &[
      // far side (0.0, 0.0, 1.0)
      vertex([-1.0, -1.0, 1.0]),
      vertex([1.0, -1.0, 1.0]),
      vertex([1.0, 1.0, 1.0]),
      vertex([-1.0, 1.0, 1.0]),
      // near side (0.0, 0.0, -1.0)
      vertex([-1.0, 1.0, -1.0]),
      vertex([1.0, 1.0, -1.0]),
      vertex([1.0, -1.0, -1.0]),
      vertex([-1.0, -1.0, -1.0]),
      // right side (1.0, 0.0, 0.0)
      vertex([1.0, -1.0, -1.0]),
      vertex([1.0, 1.0, -1.0]),
      vertex([1.0, 1.0, 1.0]),
      vertex([1.0, -1.0, 1.0]),
      // left side (-1.0, 0.0, 0.0)
      vertex([-1.0, -1.0, 1.0]),
      vertex([-1.0, 1.0, 1.0]),
      vertex([-1.0, 1.0, -1.0]),
      vertex([-1.0, -1.0, -1.0]),
      // top (0.0, 1.0, 0.0)
      vertex([1.0, 1.0, -1.0]),
      vertex([-1.0, 1.0, -1.0]),
      vertex([-1.0, 1.0, 1.0]),
      vertex([1.0, 1.0, 1.0]),
      // bottom (0.0, -1.0, 0.0)
      vertex([1.0, -1.0, 1.0]),
      vertex([-1.0, -1.0, 1.0]),
      vertex([-1.0, -1.0, -1.0]),
      vertex([1.0, -1.0, -1.0]),
    ];

    let index_data: &[u32] = &[
      0, 1, 2, 2, 3, 0, // far
      4, 5, 6, 6, 7, 4, // near
      8, 9, 10, 10, 11, 8, // right
      12, 13, 14, 14, 15, 12, // left
      16, 17, 18, 18, 19, 16, // top
      20, 21, 22, 22, 23, 20, // bottom
    ];

    rend3::types::MeshBuilder::new(vertex_positions.to_vec(), rend3::types::Handedness::Right)
      .with_indices(index_data.to_vec())
      .build()
      .unwrap()
  }

  pub fn surface(faces:&Vec<[Point; 3]>)->Mesh {
    let mut vertexes:Vec<Vec3> = vec!();
    let mut indeces:Vec<u32> = vec!();
    let mut index = 0;
    for face in faces {
      let a = face[0];
      let b = face[1];
      let c = face[2];
      vertexes.push([a.x(), a.y(), a.z()].into());
      vertexes.push([b.x(), b.y(), b.z()].into());
      vertexes.push([c.x(), c.y(), c.z()].into());
      indeces.push(index);
      indeces.push(index+1);
      indeces.push(index+2);
      index += 3;
    }
    rend3::types::MeshBuilder::new(vertexes, rend3::types::Handedness::Right)
      .with_indices(indeces)
      .build()
      .unwrap()

  }
}
