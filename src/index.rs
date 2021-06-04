use std::convert::TryFrom;
use std::ops::RangeBounds;

use anyhow::{Context as _, Result};
use cfg_if::cfg_if;
use js_sys::{Uint16Array, Uint32Array};
use web_sys::{WebGlBuffer, WebGlRenderingContext};

use crate::{resolve_range, BufferDataUsage, Context, RenderPrimitiveType};

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
}
