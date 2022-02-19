#![feature(unboxed_closures)]
#![feature(fn_traits)]
#![feature(portable_simd)]
#![feature(adt_const_params)]
#![allow(incomplete_features)]

mod color;
mod mirror;
mod shape;

pub use color::Color;
pub use mirror::{Mirror,start};
