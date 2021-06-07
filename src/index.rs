use std::ops::{self, RangeBounds};

use anyhow::{Context as _, Result};
use cfg_if::cfg_if;
use js_sys::{Uint16Array, Uint32Array};
use web_sys::{WebGlBuffer, WebGlRenderingContext};

use crate::{resolve_range, Buffer, BufferDataUsage, Context, Program, RenderPrimitiveType};

/// Stores the indices of a buffer.
pub struct Indices {
    buffer: WebGlBuffer,
    len: usize,
    ty: u32,
}

impl Indices {
    /// Allocates a buffer to store indices for a buffer of up to 65536 vertices.
    pub fn new(context: &Context, indices: &[u16], usage: BufferDataUsage) -> Result<Self> {
        let gl = &context.native;
        let buffer = gl
            .create_buffer()
            .context("Failed to allocate WebGL buffer")?;
        gl.bind_buffer(WebGlRenderingContext::ELEMENT_ARRAY_BUFFER, Some(&buffer));

        let array = Uint16Array::from(indices);
        gl.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,
            &array,
            usage.to_const(),
        );

        Ok(Self {
            buffer,
            len: indices.len(),
            ty: WebGlRenderingContext::UNSIGNED_SHORT,
        })
    }

    /// Allocates a buffer to store indices that can exceed 65536 vertices.
    ///
    /// This always fails on browsers that do not support the
    /// [`OES_element_index_uint`](https://developer.mozilla.org/en-US/docs/Web/API/OES_element_index_uint) extension.
    pub fn new_with_usize(
        context: &Context,
        indices: &[usize],
        usage: BufferDataUsage,
    ) -> Result<Self> {
        let gl = &context.native;
        gl.get_extension("OES_element_index_uint")
            .ok()
            .flatten()
            .context("Failed to enable extension for u32 element index")?;

        let array;
        cfg_if! {
            if #[cfg(target_pointer_width = "32")] {
                // In wasm32-unknown-unknown, usize == u32.
                // Let's try to optimize this majority use case.

                let indices = unsafe {
                    std::slice::from_raw_parts(indices.as_ptr() as *const u32, indices.len())
                };
                array = Uint32Array::from(indices);
            } else {
                use std::convert::TryFrom;
                let indices: Vec<u32> = indices.iter().map(|&v| u32::try_from(v).expect("Index is unreasonably large")).collect();
                array = Uint32Array::from(&indices[..]);
            }
        };

        let buffer = gl
            .create_buffer()
            .context("Failed ot allocate WebGL buffer")?;
        gl.bind_buffer(WebGlRenderingContext::ELEMENT_ARRAY_BUFFER, Some(&buffer));
        gl.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,
            &array,
            usage.to_const(),
        );

        Ok(Self {
            buffer,
            len: indices.len(),
            ty: WebGlRenderingContext::UNSIGNED_INT,
        })
    }

    /// Calls the draw operation on a
    pub(crate) fn draw(
        &self,
        mode: RenderPrimitiveType,
        context: &Context,
        items: impl RangeBounds<usize>,
    ) {
        let gl = &context.native;

        let (start, end) = resolve_range(items, self.len);

        gl.bind_buffer(
            WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,
            Some(&self.buffer),
        );
        gl.draw_elements_with_i32(mode.to_const(), end - start, self.ty, start);
    }

    /// Creates a subindex that implements [`AbstractIndices`](AbstractIndices).
    pub fn subindex<B: RangeBounds<usize> + Copy>(&self, bounds: B) -> SubIndices<'_, B> {
        SubIndices {
            indices: self,
            bounds,
        }
    }
}

/// A contiguous subsequence of an [`Indices`][Indices] buffer,
/// used to implement [`AbstractIndices`][AbstractIndices].
pub struct SubIndices<'t, B: RangeBounds<usize> + Copy> {
    indices: &'t Indices,
    bounds: B,
}

/// Types implementing this trait can be used to specify which vertices of a buffer to draw.
pub trait AbstractIndices {
    /// Draws the vertices in `buffer` indexed by `self`.
    ///
    /// Call [`Program::use_program`][Program::use_program] before calling this method.
    ///
    /// This method does not reassign uniforms.
    /// Use the `with_uniforms` method (derived by the [`Program`][super::Program] macro)
    /// to draw with uniforms specified.
    fn draw<P: Program>(
        &self,
        mode: RenderPrimitiveType,
        context: &Context,
        program: &P,
        buffer: &Buffer<P::AttrStruct>,
    );
}

impl AbstractIndices for Indices {
    fn draw<P: Program>(
        &self,
        mode: RenderPrimitiveType,
        context: &Context,
        program: &P,
        buffer: &Buffer<P::AttrStruct>,
    ) {
        program.apply_attrs(context, buffer);
        self.draw(mode, context, ..);
    }
}

impl<'t, B: RangeBounds<usize> + Copy> AbstractIndices for SubIndices<'t, B> {
    fn draw<P: Program>(
        &self,
        mode: RenderPrimitiveType,
        context: &Context,
        program: &P,
        buffer: &Buffer<P::AttrStruct>,
    ) {
        program.apply_attrs(context, buffer);
        self.indices.draw(mode, context, self.bounds);
    }
}

macro_rules! impl_bounds {
    ($ty:ty) => {
        impl AbstractIndices for $ty {
            fn draw<P: Program>(
                &self,
                mode: RenderPrimitiveType,
                context: &Context,
                program: &P,
                buffer: &Buffer<P::AttrStruct>,
            ) {
                program.apply_attrs(context, buffer);
                let (start, end) = resolve_range(self.clone(), buffer.count);
                context.native.draw_arrays(mode.to_const(), start, end);
            }
        }
    };
}

impl_bounds!(ops::Range<usize>);
impl_bounds!(ops::RangeFrom<usize>);
impl_bounds!(ops::RangeFull);
impl_bounds!(ops::RangeInclusive<usize>);
impl_bounds!(ops::RangeTo<usize>);
impl_bounds!(ops::RangeToInclusive<usize>);

impl<'t, T: AbstractIndices> AbstractIndices for &'t T {
    fn draw<P: Program>(
        &self,
        mode: RenderPrimitiveType,
        context: &Context,
        program: &P,
        buffer: &Buffer<P::AttrStruct>,
    ) {
        (&**self).draw(mode, context, program, buffer);
    }
}
