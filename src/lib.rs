//! [![GitHub actions](https://github.com/SOF3/willow/workflows/CI/badge.svg)](https://github.com/SOF3/willow/actions?query=workflow%3ACI)
//! [![crates.io](https://img.shields.io/crates/v/willow.svg)](https://crates.io/crates/willow)
//! [![crates.io](https://img.shields.io/crates/d/willow.svg)](https://crates.io/crates/willow)
//! [![docs.rs](https://docs.rs/willow/badge.svg)](https://docs.rs/willow)
//! [![GitHub](https://img.shields.io/github/last-commit/SOF3/willow)](https://github.com/SOF3/willow)
//! [![GitHub](https://img.shields.io/github/stars/SOF3/willow?style=social)](https://github.com/SOF3/willow)
//!
//! Willow is a library for using the WebGL API in WebAssembly projects.
//! It generates type-safe wrappers for WebAssembly programs using a macro syntax.

#![warn(missing_docs)]

use std::marker::PhantomData;
use std::mem;

pub use willow_codegen::Program;

#[doc(hidden)]
pub use field_offset::offset_of;
#[doc(hidden)]
pub use paste::paste;
#[doc(hidden)]
pub use typed_builder::TypedBuilder;
#[doc(hidden)]
pub use web_sys::{
    WebGlBuffer, WebGlProgram, WebGlRenderingContext, WebGlShader, WebGlUniformLocation,
};

mod types;
pub use types::*;

mod program;
pub use program::*;

mod traits;
pub use traits::*;

/// A wrapper for a WebGL rendering context.
pub struct Context {
    #[doc(hidden)]
    pub native: WebGlRenderingContext,
}

/// This macro allows efficient batch creation of programs by compiling and linking in parallel.
///
/// Example:
/// ```ignore
/// let (foo, bar, qux) = create_programs![context => Foo, Bar, Qux];
/// ```
///
/// This is more efficient than
/// ```ignore
/// let foo = Foo::create(context);
/// let bar = Bar::create(context);
/// let qux = Qux::create(context);
/// ```
#[macro_export]
macro_rules! create_programs {
    ($context:expr => $($ty:ty),* $(,)?) => {
        #[allow(non_snake_case)]
        {
            $crate::paste! {
                ($(
                    let [<var_ $ty>] = $ty::create_internally(&context);
                )*);
                $(
                    [<var_ $ty>].compile_shaders(&context);
                )*
                $(
                    [<var_ $ty>].link_shaders(&context);
                )*

                ($(
                    [<var_ $ty>],
                )*)
            }
        }
    }
}

/// Wraps a WebGL buffer.
pub struct Buffer<T: AttrStruct> {
    #[doc(hidden)]
    pub buf: WebGlBuffer,
    count: usize, // number of elements
    _ph: PhantomData<*const T>,
}

impl<T: AttrStruct> Buffer<T> {
    /// Allocates a WebGL buffer with the contents in `slice`.
    pub fn from_slice(context: &Context, slice: &[T], usage: BufferDataUsage) -> Self {
        let gl = &context.native;

        let buf = gl.create_buffer().expect("Failed to create WebGL buffer");
        gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&buf));

        let bytes = unsafe { std::slice::from_raw_parts(slice.as_ptr() as *const u8, slice.len()) };
        gl.buffer_data_with_u8_array(WebGlRenderingContext::ARRAY_BUFFER, bytes, usage.to_const());

        Self {
            buf,
            count: slice.len(),
            _ph: PhantomData,
        }
    }

    /// Binds the buffer to a specified attribute.
    pub fn bind_to_attr(&self, context: &Context, attr_index: u32, field_index: usize) {
        context.native.vertex_attrib_pointer_with_i32(
            attr_index,
            T::field_num_comps(field_index) as i32, // component count
            T::field_type(field_index),             // type
            T::field_normalized(field_index),       // normalized
            mem::align_of::<T>() as i32,            // stride
            T::field_offset(field_index) as i32,    // offset
        );
    }
}

/// The `usage` parameter passed to `bufferData`.
///
/// Corresponds to the [`usage` parameter in `bufferData`][mdn].
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/WebGLRenderingContext/bufferData#parameters
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferDataUsage {
    /// The contents are intended to be specified once by the application, and used many times as the source for WebGL drawing and image specification commands.
    StaticDraw,
    /// The contents are intended to be respecified repeatedly by the application, and used many times as the source for WebGL drawing and image specification commands.
    DynamicDraw,
    /// The contents are intended to be specified once by the application, and used at most a few times as the source for WebGL drawing and image specification commands.
    StreamDraw,
}

impl BufferDataUsage {
    fn to_const(self) -> u32 {
        match self {
            Self::StaticDraw => WebGlRenderingContext::STATIC_DRAW,
            Self::DynamicDraw => WebGlRenderingContext::DYNAMIC_DRAW,
            Self::StreamDraw => WebGlRenderingContext::STREAM_DRAW,
        }
    }
}

/// The type of rendering primitive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderPrimitiveType {
    /// Draws a single dot.
    Points,
    /// Draws a straight line to the next vertex.
    LineStrip,
    /// Draws a straight line to the next vertex, and connects the last vertex back to the first.
    LineLoop,
    /// Draws a line between a pair of vertices.
    Lines,
    /// <https://en.wikipedia.org/wiki/Triangle_strip>
    TriangleStrip,
    /// <https://en.wikipedia.org/wiki/Triangle_fan>
    TriangleFan,
    /// Draws a triangle for a group of three vertices.
    Triangles,
}

impl RenderPrimitiveType {
    fn to_const(self) -> u32 {
        match self {
            Self::Points => WebGlRenderingContext::POINTS,
            Self::LineStrip => WebGlRenderingContext::LINE_STRIP,
            Self::LineLoop => WebGlRenderingContext::LINE_LOOP,
            Self::Lines => WebGlRenderingContext::LINES,
            Self::TriangleStrip => WebGlRenderingContext::TRIANGLE_STRIP,
            Self::TriangleFan => WebGlRenderingContext::TRIANGLE_FAN,
            Self::Triangles => WebGlRenderingContext::TRIANGLES,
        }
    }
}
