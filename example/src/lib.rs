//! Example webapp using [`willow`](https://docs.rs/willow/).

#![allow(clippy::blacklisted_name)]
#![warn(missing_docs)]

use nalgebra::{Matrix4, Vector3};
use wasm_bindgen::prelude::*;
use web_sys::WebGlRenderingContext;
use willow::{
    AspectFix, Attribute, BufferDataUsage, Clear, Context, Indices, Program, ProgramData,
    RenderPrimitiveType, Uniform,
};

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

    let context = Context::from_canvas(canvas, AspectFix::FromWidth).unwrap();
    let (foo,) = willow::create_programs!(context => Foo);

    context.clear(Clear {
        color: Some([0., 0., 0., 1.]),
        depth: Some(1.),
        stencil: None,
    });
    context.native.enable(WebGlRenderingContext::DEPTH_TEST);
    context.native.depth_func(WebGlRenderingContext::LEQUAL);

    let attrs = Foo::prepare_buffer(
        &context,
        &[
            FooAttr {
                a_offset: Vector3::new(1.0, 0.0, 0.0),
                a_color: Vector3::new(1.0, 0.0, 0.0),
            },
            FooAttr {
                a_offset: Vector3::new(-1.0, -1.0, 0.0),
                a_color: Vector3::new(1.0, 1.0, 0.0),
            },
            FooAttr {
                a_offset: Vector3::new(-1.0, 1.0, 0.0),
                a_color: Vector3::new(0.0, 1.0, 0.0),
            },
            FooAttr {
                a_offset: Vector3::new(0.0, 0.0, 1.0),
                a_color: Vector3::new(0.0, 0.0, 1.0),
            },
        ],
        BufferDataUsage::StaticDraw,
    );

    let indices = Indices::new(
        &context,
        &[0, 1, 2, 0, 1, 3, 1, 2, 3, 2, 0, 3],
        BufferDataUsage::StaticDraw,
    )
    .unwrap();

    foo.with_uniforms()
        .u_alpha(1.0)
        .u_transform(
            Matrix4::new_perspective(context.aspect(), 1.5, 0.01, 5.)
                * nalgebra::Rotation::from_euler_angles(0.5, 0.5, 0.5)
                    .to_homogeneous()
                    .append_translation(&Vector3::new(0., 0., -2.)),
        )
        .draw(&context, RenderPrimitiveType::Triangles, &attrs, &indices)
        .unwrap();
}
