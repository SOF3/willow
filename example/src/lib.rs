//! Example webapp using [`willow`](https://docs.rs/willow/).

#![allow(clippy::blacklisted_name)]
#![warn(missing_docs)]

use nalgebra::{Matrix4, Vector3};
use wasm_bindgen::prelude::*;
use willow::{Attribute, Context, Program, ProgramData, Uniform};

/// This type wraps the program with the `foo.vert` and `foo.frag` shaders.
#[derive(Program)]
#[willow(path = "foo")]
pub struct Foo {
    data: ProgramData,
    u_alpha: Uniform<f32>,
    u_transform: Uniform<Matrix4<f32>>,
    a_color: Attribute<Vector3<f32>>,
    a_offset: Attribute<Vector3<f32>>,
}

/// WebGL entry function.
#[wasm_bindgen(start)]
pub fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    wasm_logger::init(wasm_logger::Config::default());

    let canvas = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .get_element_by_id("canvas")
        .unwrap();

    let context = Context::from_canvas(canvas).unwrap();
    let (foo,) = willow::create_programs!(context => Foo);
}
