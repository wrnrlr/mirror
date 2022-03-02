use bevy::prelude::*;
use g3::*;
use crate::{DoubleSidedPlane, pan_orbit_camera, Rgba, spawn_camera};

pub struct PlotPlugin;

impl Plugin for PlotPlugin {
  fn build(&self, app: &mut App) {
    app.add_startup_system(setup_light)
       .add_startup_system(spawn_camera)
       .add_system(points_added)
       .add_system(points_changed)
       .add_system(lines_added)
       .add_system(plane_added)
       .add_system(pan_orbit_camera);
  }

  fn name(&self) -> &str {
    "Plot3DPlugin"
  }
}

const LINE_RADIUS:f32 = 0.05;
const POINT_RADIUS:f32 = 0.1;

fn setup_light(mut cmd: Commands) {
  cmd.spawn_bundle(PointLightBundle {
    point_light: PointLight {intensity: 1500.0, shadows_enabled: false, ..Default::default()},
    transform: Transform::from_xyz(4.0, 8.0, 4.0), ..Default::default()
  });
}

fn points_added(
  mut cmd:Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
  q:Query<(Entity, &Point, &Rgba, Added<Point>)>
) {
  for (e,p,c,_) in q.iter() {
    cmd.entity(e).insert_bundle(PbrBundle {
      mesh: meshes.add(Mesh::from(shape::Icosphere { radius: POINT_RADIUS, subdivisions: 8 })),
      material: materials.add(Color::rgb(c.red(), c.green(), c.blue()).into()),
      transform: Transform::from_xyz(p.x(), p.y(), p.z()),
      ..Default::default()
    });
  }
}

fn points_changed(mut q:Query<(&Point, &mut Transform, Changed<Point>)>) {
  for (p,mut t,_) in q.iter_mut() {
    *t.translation = *Vec3::from([p.x(), p.y(), p.z()]);
  }
}

fn lines_added(
  mut cmd:Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
  q:Query<(Entity, &Line, &Rgba, Added<Line>), (Without<Handle<Mesh>>)>
) {
  for (e,l,c,_) in q.iter() {
    let b:Branch = l.into();
    // let r:Rotor = b.into();
    // let ea:EulerAngles = r.into();

    cmd.entity(e).insert_bundle(PbrBundle {
      mesh: meshes.add(Mesh::from(shape::Capsule { radius: LINE_RADIUS, depth: 2.0, rings: 1, ..Default::default()})),
      material: materials.add(Color::rgb(c.red(), c.green(), c.blue()).into()),
      transform: Transform::from_rotation(Quat::from_array([b.x(), b.y(), b.z(), 0.0])),
      // transform: Transform::from_rotation(Quat::from_euler(glam::EulerRot::XYZ, ea.roll, ea.pitch, ea.yaw)),
      ..Default::default()
    });
  }
}

fn plane_added(
  mut cmd:Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
  q:Query<(Entity, &Plane, &Rgba, Added<Plane>), (Without<Handle<Mesh>>)>
) {
  for (e,p,c,_) in q.iter() {
    print!("add plane ");
    cmd.entity(e).insert_bundle(PbrBundle {
      mesh: meshes.add(Mesh::from(DoubleSidedPlane{ size: 1.0 })),
      material: materials.add(Color::rgb(c.red(), c.green(), c.blue()).into()),
      // transform: Transform::from_rotation(Quat::from_array([p.x(), p.y(), b.z(), 0.0])),
      // transform: Transform::from_rotation(Quat::from_euler(glam::EulerRot::XYZ, ea.roll, ea.pitch, ea.yaw)),
      ..Default::default()
    });
  }
}

// fn line_changed(
//   mut q:Query<(&Line, &mut Transform, Changed<Point>)>
// ) {
//   for (p,mut t,_) in q.iter_mut() {
//     *t.translation = *Vec3::from([p.x(), p.y(), p.z()]);
//   }
// }

// fn edge_added(
//   mut cmd:Commands,
//   mut meshes: ResMut<Assets<Mesh>>,
//   mut materials: ResMut<Assets<StandardMaterial>>,
//   q:Query<(Entity, &(Point,Point), &Rgba, Added<Point>)>
// ) {
//   for (e,p,c,_) in q.iter() {
//     cmd.entity(e).insert_bundle(PbrBundle {
//       mesh: meshes.add(Mesh::from(shape::Icosphere { radius: 0.1, subdivisions: 8 })),
//       material: materials.add(Color::rgb(c.red(), c.green(), c.blue()).into()),
//       transform: Transform::from_xyz(p.x(), p.y(), p.z()),
//       ..Default::default()
//     });
//   }
// }
//
// fn edge_changed(
//   mut q:Query<(&(Point,Point), &mut Transform, Changed<Point>)>
// ) {
//   for (p,mut t,_) in q.iter_mut() {
//     *t.translation = *Vec3::from([p.x(), p.y(), p.z()]);
//   }
// }
