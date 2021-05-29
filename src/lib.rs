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

pub use willow_codegen::Program;

#[doc(hidden)]
pub use paste::paste;
#[doc(hidden)]
pub use typed_builder::TypedBuilder;
#[doc(hidden)]
pub use web_sys::{WebGlBuffer, WebGlProgram, WebGlRenderingContext, WebGlShader};

mod uniform;
pub use uniform::*;

/// A wrapper for a WebGL rendering context.
pub struct Context {
    native: WebGlRenderingContext,
}

pub struct ProgramData {
    #[doc(hidden)]
    pub program: WebGlProgram,
    #[doc(hidden)]
    pub vertex_shader: WebGlShader,
    #[doc(hidden)]
    pub fragment_shader: WebGlShader,
}

/// Represents WebGL programs.
///
/// This type should only be implemented by the [`Program`](derive.Porgram.html) macro.
pub trait Program: Sized {
    /// The struct generated for storing attributes
    type AttrStruct: ProgramAttrStruct;

    /// Compiles and links the program in the given [`Context`](struct.Context.html).
    fn create(context: &Context) -> Self {
        let gl = &context.native;

        let p = Self::create_internally(gl);
        p.compile_shaders(gl);
        p.link_shaders(gl);
        p
    }

    /// Creates an instance of the type. Allocate necessary resources like `gl.createShader()`.
    #[doc(hidden)]
    fn create_internally(gl: &WebGlRenderingContext) -> Self;

    /// Compiles the vertex and fragment shaders.
    #[doc(hidden)]
    fn compile_shaders(&self, gl: &WebGlRenderingContext);

    /// Attaches and links the vertex and fragment shaders.
    #[doc(hidden)]
    fn link_shaders(&self, gl: &WebGlRenderingContext);

    /// Prepares a buffer with the attributes in the vec.
    fn prepare_buffer(context: &Context, attrs: Vec<Self::AttrStruct>) -> Buffer<Self::AttrStruct> {
        todo!()
    }

    /// Runs the program with the given attributes.
    fn with_attributes(&self, buffer: Buffer<Self::AttrStruct>);
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
                    let [<var_ $ty>] = $ty::create_internally(&context.native);
                )*);
                $(
                    [<var_ $ty>].compile_shaders(&context.native);
                )*
                $(
                    [<var_ $ty>].link_shaders(&context.native);
                )*

                ($(
                    [<var_ $ty>],
                )*)
            }
        }
    }
}

pub trait ProgramAttrStruct {
    /// The number of fields in the struct
    fn fields_count() -> usize;

    /// The offset of field `i` in the struct in bytes
    fn field_offset(i: usize) -> usize;
}

pub struct Buffer<T: ProgramAttrStruct> {
    buf: WebGlBuffer,
    _ph: PhantomData<*const T>,
}

pub struct Attribute {}

impl Attribute {
    #[doc(hidden)]
    pub fn create_from_macro() -> Self {
        Self {}
    }
}

pub struct Uniform<T: UniformType> {
    _ph: PhantomData<&'static T>,
}

impl<T: UniformType> Uniform<T> {
    #[doc(hidden)]
    pub fn create_from_macro() -> Self {
        Self { _ph: PhantomData }
    }
}
