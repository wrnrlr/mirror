use bevy::prelude::*;
use g3::{Point,point};
use crate::Rgba;

pub struct PlotPlugin;

impl Plugin for PlotPlugin {
  fn build(&self, app: &mut App) {
    app.add_startup_system(setup)
       .add_system(g3_points)
       .add_system(points_changed);
  }

  fn name(&self) -> &str {
    "Plot3DPlugin"
  }
}

/// set up a simple 3D scene
fn setup(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
) {
  commands.spawn_bundle(PointLightBundle {
    point_light: PointLight {
      intensity: 1500.0,
      shadows_enabled: false,
      ..Default::default()
    },
    transform: Transform::from_xyz(4.0, 8.0, 4.0), ..Default::default()
  });
  // camera
  commands.spawn_bundle(PerspectiveCameraBundle {
    transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ..Default::default()
  });
}

fn g3_points(
  mut cmd:Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
  q:Query<Entity, (With<Point>, With<Rgba>)>
) {
  for e in q.iter() {
    let (p,c) = e;
    cmd.entity(e).insert(
      PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Icosphere { radius: 0.1, subdivisions: 8 })),
        material: materials.add(Color::rgb(c.red(), c.green(), c.blue()).into()),
        transform: Transform::from_xyz(p.x(), p.y(), p.z()),
        ..Default::default()
      }
    );
    // cmd.spawn_bundle();
  }
}

fn points_changed(
  mut cmd:Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
  q:Query<(&Point, &Rgba), Changed<Point>>
) {
  for e in q.iter() {
    println!("changed p {}", e);
  }
}