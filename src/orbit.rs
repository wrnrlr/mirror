use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;

/// Tags an entity as capable of panning and orbiting.
#[derive(Component)]
pub struct PanOrbitCamera {
  /// The "focus point" to orbit around. It is automatically updated when panning the camera
  pub focus: Vec3,
  pub radius: f32,
  pub upside_down: bool,
}

impl Default for PanOrbitCamera {
  fn default() -> Self {
    PanOrbitCamera {
      focus: Vec3::ZERO,
      radius: 5.0,
      upside_down: false,
    }
  }
}

/// Pan the camera with middle mouse click, zoom with scroll wheel, orbit with right mouse click.
pub fn pan_orbit_camera(
  windows: Res<Windows>,
  mut ev_motion: EventReader<MouseMotion>,
  mut ev_scroll: EventReader<MouseWheel>,
  input_mouse: Res<Input<MouseButton>>,
  mut query: Query<(&mut PanOrbitCamera, &mut Transform, &PerspectiveProjection)>,
) {
  // change input mapping for orbit and panning here
  let orbit_button = MouseButton::Right;
  let pan_button = MouseButton::Middle;

  let mut pan = Vec2::ZERO;
  let mut rotation_move = Vec2::ZERO;
  let mut scroll = 0.0;
  let mut orbit_button_changed = false;

  if input_mouse.pressed(orbit_button) {
    for ev in ev_motion.iter() {
      rotation_move += ev.delta;
    }
  } else if input_mouse.pressed(pan_button) {
    // Pan only if we're not rotating at the moment
    for ev in ev_motion.iter() {
      pan += ev.delta;
    }
  }
  for ev in ev_scroll.iter() {
    scroll += ev.y;
  }
  if input_mouse.just_released(orbit_button) || input_mouse.just_pressed(orbit_button) {
    orbit_button_changed = true;
  }

  for (mut pan_orbit, mut transform, projection) in query.iter_mut() {
    if orbit_button_changed {
      // only check for upside down when orbiting started or ended this frame
      // if the camera is "upside" down, panning horizontally would be inverted, so invert the input to make it correct
      let up = transform.rotation * Vec3::Y;
      pan_orbit.upside_down = up.y <= 0.0;
    }

    let mut any = false;
    if rotation_move.length_squared() > 0.0 {
      any = true;
      let window = get_primary_window_size(&windows);
      let delta_x = {
        let delta = rotation_move.x / window.x * std::f32::consts::PI * 2.0;
        if pan_orbit.upside_down { -delta } else { delta }
      };
      let delta_y = rotation_move.y / window.y * std::f32::consts::PI;
      let yaw = Quat::from_rotation_y(-delta_x);
      let pitch = Quat::from_rotation_x(-delta_y);
      transform.rotation = yaw * transform.rotation; // rotate around global y axis
      transform.rotation = transform.rotation * pitch; // rotate around local x axis
    } else if pan.length_squared() > 0.0 {
      any = true;
      // make panning distance independent of resolution and FOV,
      let window = get_primary_window_size(&windows);
      pan *= Vec2::new(projection.fov * projection.aspect_ratio, projection.fov) / window;
      // translate by local axes
      let right = transform.rotation * Vec3::X * -pan.x;
      let up = transform.rotation * Vec3::Y * pan.y;
      // make panning proportional to distance away from focus point
      let translation = (right + up) * pan_orbit.radius;
      pan_orbit.focus += translation;
    } else if scroll.abs() > 0.0 {
      any = true;
      pan_orbit.radius -= scroll * pan_orbit.radius * 0.2;
      // dont allow zoom to reach zero or you get stuck
      pan_orbit.radius = f32::max(pan_orbit.radius, 0.05);
    }

    if any {
      // emulating parent/child to make the yaw/y-axis rotation behave like a turntable
      // parent = x and y rotation
      // child = z-offset
      let rot_matrix = Mat3::from_quat(transform.rotation);
      transform.translation = pan_orbit.focus + rot_matrix.mul_vec3(Vec3::new(0.0, 0.0, pan_orbit.radius));
    }
  }
}

fn get_primary_window_size(windows: &Res<Windows>) -> Vec2 {
  let window = windows.get_primary().unwrap();
  let window = Vec2::new(window.width() as f32, window.height() as f32);
  window
}

/// Spawn a camera like this
pub fn spawn_camera(mut commands: Commands) {
  let translation = Vec3::new(-2.0, 2.5, 5.0);
  let radius = translation.length();

  commands.spawn_bundle(PerspectiveCameraBundle {
    transform: Transform::from_translation(translation)
      .looking_at(Vec3::ZERO, Vec3::Y),
    ..Default::default()
  }).insert(PanOrbitCamera {
    radius,
    ..Default::default()
  });
}


// https://github.com/Jam3/orbit-controls
// https://catlikecoding.com/unity/tutorials/movement/orbit-camera/
// https://observablehq.com/@enkimute/glu-lookat-in-3d-pga#11

// https://sci-hub.yncjkj.com/10.1016/j.cag.2004.04.007
// https://sci-hub.yncjkj.com/10.1109/mcg.2003.1185582

// Calculate normals
// https://www.cv.nrao.edu/~mmorgan2/resources/geo3.html
// https://www.ljll.math.upmc.fr/~frey/papers/scientific%20visualisation/Zaharia%20M.D.,%20Dorst%20L.,%20Modeling%20and%20visualization%20of%203D%20polygonal%20mesh%20surfaces%20using%20geometric%20algebra.pdf
// https://en.wikipedia.org/wiki/Comparison_of_vector_algebra_and_geometric_algebra#Determinant_expansion_of_cross_and_wedge_products
// https://www.khronos.org/opengl/wiki/Calculating_a_Surface_Normal

// pub fn cylinder(_streams:super::Streams, radius:f32, height:f32)->Geometry {
//   const RADIAL_SEGMENTS:u32 = 8;
//
//   let half_height = height / 2 as f32;
//
//   let mut positions = Vec::new();
//
//
//   for x in 1..RADIAL_SEGMENTS {
//     let theta = x as f32 / RADIAL_SEGMENTS as f32;
//     let sin_theta = theta.sin();
//     let cos_theta = theta.cos();
//
//     // vertex
//     positions.push(Position([radius*sin_theta, height + half_height, radius * cos_theta]));
//
//   }
//
//   println!("Positions: {:?}", positions);
//
//   Geometry{
//     positions: vec!(Position([0f32, 0.0, 0.0])),
//     normals: None, indices: None, radius
//   }
//
// }