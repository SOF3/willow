#![cfg_attr(debug_assertions, allow(dead_code))]

extern crate proc_macro;

use proc_macro2::TokenStream;
use syn::Result;

mod gen;
mod parse;

/// Derives a user-friendly wrapper for `WebGlProgram` from a struct.
///
/// The struct must only contain the following fields:
/// - Exactly one `ProgramData` field
/// - `Uniform<T>` fields (use `#[willow(uniform(T)]` if aliased)
/// - `Attribute<T>` fields (use `#[willow(attribute(T))]` if aliased)
///
/// In the struct attribute, the path to the GLSL shaders must be specified:
/// ```ignore
/// #[willow(path = "scene")]
/// ```
///
/// This will load the GLSL shaders by running `include_str!("scene.vert")` and
/// `include_str!("scene.frag")`.
///
/// If they are already loaded in a constant, write this instead:
/// ```ignore
/// #[willow(vert = VERTEX_SHADER_CODE, frag = FRAGMENT_SHADER_CODE)]
/// ```
///
/// # Example
/// ```ignore
/// #[derive(willow::Program)]
/// #[willow(path = "scene")]
/// struct Scene {
///     vertices: Attribute<Vector3<f32>>,
///     normals: Attribute<Vector3<f32>>,
///     projection: Uniform<Matrix4<f32>>,
/// }
/// ```
///
/// With the files `scene.vert` and `scene.frag` containing at least these declarations:
/// ```glsl
/// attribute vec3 vertices;
/// attribute vec3 normals;
/// uniform mat4 projection;
/// ```
///
/// Then it can be used like this:
/// ```ignore
/// fn render(
///     gl: &WebGlRenderingContext,
///     scene: &Scene,
///     buffer: &Buffer,
///     projection: Matrix,
/// ) {
///     scene.call()
///         .vertices(vertices)
///         .normals(normals)
///         .projection(projection)
///         .draw_indexed(WebGlRenderingContext::TRIANGLES, indices);
/// }
/// ```
#[proc_macro_derive(Program, attributes(willow))]
pub fn program(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    program_imp(input.into())
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

fn program_imp(input: TokenStream) -> Result<TokenStream> {
    let info = parse::parse_input(input)?;

    Ok(gen::gen_code(&info))
}
