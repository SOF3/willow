//! Types in this module are used as fields in the [`Program`][super::Program] impl
//! to hold resources allocated from the `WebGlRenderingContext`.

use std::marker::PhantomData;

use once_cell::unsync::OnceCell;
use web_sys::{WebGlUniformLocation, WebGlShader, WebGlProgram};

use crate::{Context, AttributeType, UniformType};

/// An internal type used to hold program-specific resources.
/// There must be exactly one field in a [`Program`][super::Program]-deriving struct
/// holding `ProgramData` as the value.
pub struct ProgramData {
    #[doc(hidden)]
    pub program: WebGlProgram,
    #[doc(hidden)]
    pub vertex_shader: WebGlShader,
    #[doc(hidden)]
    pub fragment_shader: WebGlShader,
}

/// In a [`Program`][super::Program]-deriving struct,
/// a field of type `Attribute<T>` indicates that
/// the vertex shader has an attribute with the type compatible with `T`.
pub struct Attribute<T: AttributeType> {
    location: OnceCell<u32>,
    _ph: PhantomData<&'static T>,
}

impl<T: AttributeType> Attribute<T> {
    /// Internal method used to create a raw `Attribute` value.
    #[doc(hidden)]
    pub fn create_from_macro() -> Self {
        Self {
            location: OnceCell::new(),
            _ph: PhantomData,
        }
    }

    /// Lazily retrieves an attribute location
    /// aod stores it in this `Attribute` struct.
    pub fn get_location(&self, context: &Context, program: &ProgramData, name: &str) -> u32 {
        *self.location.get_or_init(|| {
            let location = context.native.get_attrib_location(&program.program, name);
            let location = location as u32;
            context.native.enable_vertex_attrib_array(location);
            location
        })
    }
}

/// In a [`Program`][super::Program]-deriving struct,
/// a field of type `Uniform<T>` indicates that
/// the vertex shader has a uniform with the type compatible with `T`.
pub struct Uniform<T: UniformType> {
    location: OnceCell<Option<WebGlUniformLocation>>,
    _ph: PhantomData<&'static T>,
}

impl<T: UniformType> Uniform<T> {
    /// Internal method used to create a raw `Uniform` value.
    #[doc(hidden)]
    pub fn create_from_macro() -> Self {
        Self {
            location: OnceCell::new(),
            _ph: PhantomData,
        }
    }

    /// Lazily retrieves the location of the uniform in the program
    /// aod stores it in this `Uniform` struct.
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
