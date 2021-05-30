//! [![GitHub actions](https://github.com/SOF3/willow/workflows/CI/badge.svg)](https://github.com/SOF3/willow/actions?query=workflow%3ACI)
//! [![crates.io](https://img.shields.io/crates/v/willow.svg)](https://crates.io/crates/willow)
//! [![crates.io](https://img.shields.io/crates/d/willow.svg)](https://crates.io/crates/willow)
//! [![docs.rs](https://docs.rs/willow/badge.svg)](https://docs.rs/willow)
//! [![GitHub](https://img.shields.io/github/last-commit/SOF3/willow)](https://github.com/SOF3/willow)
//! [![GitHub](https://img.shields.io/github/stars/SOF3/willow?style=social)](https://github.com/SOF3/willow)
//!
//! Willow is a library for using the WebGL API in WebAssembly projects.
//! It generates type-safe wrappers for WebAssembly programs using a macro syntax.

use std::marker::PhantomData;
use std::mem;
use std::ops::{Bound, RangeBounds};

use once_cell::unsync::OnceCell;

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

/// A wrapper for a WebGL rendering context.
pub struct Context {
    #[doc(hidden)]
    pub native: WebGlRenderingContext,
}

/// Represents WebGL programs.
///
/// This type should only be implemented by the [`Program`](derive.Porgram.html) macro.
pub trait Program: Sized {
    /// The struct generated for storing attributes.
    /// Always `#[repr(C)]`.
    type AttrStruct: AttrStruct;

    /// Compiles and links the program in the given [`Context`](struct.Context.html).
    fn create(context: &Context) -> Self {
        let p = Self::create_internally(context);
        p.compile_shaders(context);
        p.link_shaders(context);
        p
    }

    /// Creates an instance of the type. Allocate necessary resources like `gl.createShader()`.
    fn create_internally(gl: &Context) -> Self;

    /// Compiles the vertex and fragment shaders.
    fn compile_shaders(&self, gl: &Context);

    /// Attaches and links the vertex and fragment shaders.
    fn link_shaders(&self, gl: &Context);

    /// Prepares a buffer with the attributes in the vec.
    fn prepare_buffer(
        context: &Context,
        attrs: &[Self::AttrStruct],
        usage: BufferDataUsage,
    ) -> Buffer<Self::AttrStruct> {
        Buffer::from_slice(context, attrs, usage)
    }

    /// Runs the program with the given attributes.
    fn draw_fully(
        &self,
        context: &Context,
        mode: RenderPrimitiveType,
        buffer: &Buffer<Self::AttrStruct>,
        items: impl RangeBounds<usize>,
    ) {
        let start = match items.start_bound() {
            Bound::Included(&x) => x as i32,
            Bound::Excluded(&x) => x as i32 - 1,
            Bound::Unbounded => 0,
        };
        let end = match items.end_bound() {
            Bound::Included(&x) => x as i32 + 1,
            Bound::Excluded(&x) => x as i32,
            Bound::Unbounded => buffer.count as i32,
        };

        assert!(
            end <= buffer.count as i32,
            "items range exceeds buffer size"
        );

        self.apply_attrs(context, buffer);

        context.native.draw_arrays(mode.to_const(), start, end);
    }

    fn apply_attrs(&self, context: &Context, buffer: &Buffer<Self::AttrStruct>);
}

/// This macro allows efficient batch creation of programs by compiling and linking in parallel.
///
/// Example:
/// ```
/// let (foo, bar, qux) = create_programs![context => Foo, Bar, Qux];
/// ```
///
/// This is more efficient than
/// ```
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

/// The trait implemented by attribute structs
pub trait AttrStruct {
    /// The number of fields in the struct
    fn fields_count() -> usize;

    /// The GLSL name of the attribute corresponding to field `i`
    fn field_gl_name(i: usize) -> &'static str;

    /// The offset of field `i` in the struct in bytes
    fn field_offset(i: usize) -> usize;

    /// The base type of field `i` in the struct with constants like `WebGlRenderingContext::BYTE`
    fn field_type(i: usize) -> u32;

    /// The number of components for the type in field `i`
    fn field_num_comps(i: usize) -> usize;

    /// Whether the field `i` should be normalized
    fn field_normalized(i: usize) -> bool;
}

pub struct Buffer<T: AttrStruct> {
    #[doc(hidden)]
    pub buf: WebGlBuffer,
    count: usize, // number of elements
    _ph: PhantomData<*const T>,
}

impl<T: AttrStruct> Buffer<T> {
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
    /// https://en.wikipedia.org/wiki/Triangle_strip
    TriangleStrip,
    /// https://en.wikipedia.org/wiki/Triangle_fan
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

/// An internal type used to hold program-specific resources.
pub struct ProgramData {
    #[doc(hidden)]
    pub program: WebGlProgram,
    #[doc(hidden)]
    pub vertex_shader: WebGlShader,
    #[doc(hidden)]
    pub fragment_shader: WebGlShader,
}

pub struct Attribute<T: AttributeType> {
    location: OnceCell<u32>,
    _ph: PhantomData<&'static T>,
}

impl<T: AttributeType> Attribute<T> {
    #[doc(hidden)]
    pub fn create_from_macro() -> Self {
        Self {
            location: OnceCell::new(),
            _ph: PhantomData,
        }
    }

    pub fn get_location(&self, context: &Context, program: &ProgramData, name: &str) -> u32 {
        *self.location.get_or_init(|| {
            let location = context.native.get_attrib_location(&program.program, name);
            let location = location as u32;
            context.native.enable_vertex_attrib_array(location);
            location
        })
    }
}

pub struct Uniform<T: UniformType> {
    location: OnceCell<Option<WebGlUniformLocation>>,
    _ph: PhantomData<&'static T>,
}

impl<T: UniformType> Uniform<T> {
    #[doc(hidden)]
    pub fn create_from_macro() -> Self {
        Self {
            location: OnceCell::new(),
            _ph: PhantomData,
        }
    }

    pub fn get_location<'t>(
        &'t self,
        context: &Context,
        program: &ProgramData,
        name: &str,
    ) -> Option<&'t WebGlUniformLocation> {
        self.location
            .get_or_init(|| context.native.get_uniform_location(&program.program, name))
            .as_ref()
    }
}
