extern crate proc_macro;

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Result;

mod parse;

/// Derives a user-friendly wrapper for `WebGlProgram` from a struct.
///
/// The struct must only contain the following fields:
/// - `UniformLocation<T>` fields (use `#[wglrs(uniform, T)]` if aliased)
/// - `AttrLocation` fields (use `#[wglrs(attribute)]` if aliased)
///
/// In the struct attribute, the path to the GLSL shaders must be specified:
/// ```ignore
/// #[wglrs(path = "scene")]
/// ```
///
/// This will load the GLSL shaders by running `include_str!("scene.vert")` and
/// `include_str!("scene.frag")`.
///
/// If they are already loaded in a constant, write this instead:
/// ```ignore
/// #[wglrs(vert_code = VERTEX_SHADER_CODE, vert_code = FRAGMENT_SHADER_CODE)]
/// ```
///
/// # Example
/// ```ignore
/// #[derive(wglrs::Program)]
/// #[wglrs(path = "scene")]
/// struct Scene {
///     vertices: AttrLocation,
///     normals: AttrLocation,
///     projection: UniformLocation<Matrix>,
/// }
/// ```
///
/// With the files `scene.vert` and `scene.frag` containing at least these declarations:
/// ```ignore
/// attribute vec3 vertices;
/// attribute vec3 normals;
/// uniform mat4 projection;
/// ```
///
/// Then it can be used like this:
/// ```ignore
/// fn render(
///     gl: &WebGlRenderingContext,
///     scene: &Program<Scene>,
///     vertices: &Buffer,
///     normals: &Buffer,
///     projection: Matrix,
/// ) {
///     scene.call()
///         .vertices(vertices)
///         .normals(normals)
///         .projection(projection)
///         .draw_indexed(WebGlRenderingContext::TRIANGLES, indices);
/// }
/// ```
#[proc_macro_derive(Program, attributes(wglrs))]
pub fn program(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    program_imp(input.into())
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

fn program_imp(input: TokenStream) -> Result<TokenStream> {
    let info = parse::parse_input(input)?;

    Ok(gen::gen_code(&info))
}
