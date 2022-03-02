#![feature(unboxed_closures)]
#![feature(fn_traits)]
#![feature(portable_simd)]
#![feature(adt_const_params)]
#![allow(incomplete_features)]

mod color;
mod mirror;
mod plot;
mod orbit;
mod plane;

pub use color::Rgba;
pub use plot::PlotPlugin;
pub use orbit::*;
pub(crate) use plane::DoubleSidedPlane;
