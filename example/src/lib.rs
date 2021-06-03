#![allow(clippy::blacklisted_name)]

// mod x;

use nalgebra::{Matrix4, Vector3};
use wasm_bindgen::prelude::*;
use willow::{Attribute, Context, Program, ProgramData, Uniform};

#[derive(Program)]
#[willow(path = "foo")]
pub struct Foo {
    data: ProgramData,
    u_alpha: Uniform<f32>,
    u_transform: Uniform<Matrix4<f32>>,
    a_color: Attribute<Vector3<f32>>,
    a_offset: Attribute<Vector3<f32>>,
}

#[wasm_bindgen(start)]
pub fn main() {
    let canvas = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .get_element_by_id("canvas")
        .unwrap();

    let context = Context::from_canvas(canvas).unwrap();
    let (foo,) = willow::create_programs!(context => Foo);
}
