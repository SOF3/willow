use std::ops::{Bound, RangeBounds};

use super::{Buffer, BufferDataUsage, Context, RenderPrimitiveType};

/// Represents WebGL programs.
///
/// This type should only be implemented by the [`Program`][super::Program] macro.
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

    /// Applies the buffer ot the attributes in this program.
    fn apply_attrs(&self, context: &Context, buffer: &Buffer<Self::AttrStruct>);
}

/// The trait implemented by attribute structs.
///
/// Methods in this struct describe the structure of the fields.
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