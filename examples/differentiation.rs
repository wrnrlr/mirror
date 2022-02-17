use g3::*;
use mirror::{Mirror, Color, start};

// Automatic differentiation allows you to easily evaluate derivatives of arbitrary polynomial functions.
// Unlike numerical methods, using dual numbers for this purpose is artefact free and enables machine-level
// precision. The degenerate metric of the PGA framework embeds with e01 and e02 two ideal (;)) elements to use
// for the calculation of partial derivatives. We demonstrate by graphing a two variable function and both
// its partial derivatives.

fn align(p:Point, q:Point)->Translator {
  (q.normalized()/p.normalized()).sqrt()
}

fn steps(n:u32)->impl Iterator<Item = f32> {
  (0..n).map(move|i| i as f32/(n as f32-1.0))
}

fn lerp(m:Translator, f:f32)->Translator {
  (m*f) //.normalized(); TODO normalized is broken
}

fn path(m:Translator, n:u32, x:Point)->impl Iterator<Item = Point> {
  steps(n).map(move|f|lerp(m, f)(x))
}

fn main() {
  let mut mr = Mirror::default();

  let a = point(-1.0,1.0,0.0);
  let b = point(1.0,1.0,0.0);
  let c = point(1.0,-1.0,0.0);
  let d = point(-1.0,-1.0,0.0);

  let f = |x:f32,y:f32,t:f32| 0.5*(t*5.0).sin()*x*x*x-0.5*t.cos()*y*y;

  mr.vertex(a, Color::RED);
  mr.vertex(b, Color::GREEN);
  mr.vertex(c, Color::BLUE);
  mr.vertex(d, Color::YELLOW);

  let ac = align(a,c);
  path(ac,8,a).for_each(|p|{mr.vertex(p,Color::CYAN);});

  let ab = align(a,b);
  let ad = align(a,b);

  let ad = align(a,d);
  for i in 0..8 {
    let down = lerp(ad, i as f32/(7.0));
    let p = down(a);
    let q = down(b);
    let pq = align(down(a), q);
    path(pq,8,p).for_each(|p|{mr.vertex(p,Color::CYAN);});
  }

  start(
    mr,
    winit::window::WindowBuilder::new()
      .with_title("differentiation")
      .with_maximized(true),
  );
}

