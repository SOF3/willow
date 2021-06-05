use crate::{AbstractIndices, Buffer, BufferDataUsage, Context, RenderPrimitiveType};

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

    /// Calls the WebGL context to use the current program for draw calls.
    fn use_program(&self, gl: &Context);

    /// Runs the program with the given attributes indexed by `indices`.
    ///
    /// This method is identical to [`AbstractIndices::draw`][AbstractIndices::draw],
    /// except with different parameter order.
    ///
    /// This method does not reassign uniforms.
    /// Use the `with_uniforms` method (derived by the [`Program`][super::Program] macro)
    /// to draw with uniforms specified.
    fn draw(
        &self,
        context: &Context,
        mode: RenderPrimitiveType,
        buffer: &Buffer<Self::AttrStruct>,
        indices: &impl AbstractIndices,
    ) {
        self.use_program(context);
        indices.draw(mode, context, self, buffer);
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
