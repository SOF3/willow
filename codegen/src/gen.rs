use proc_macro2::TokenStream;
use quote::quote;

use super::parse::Input;

pub fn gen_code(input: &Input) -> TokenStream {
    let imp = gen_program_impl(input);
    let attrs = gen_attrs(input);
    eprintln!("imp: {}", imp);
    eprintln!("attrs: {}", attrs);
    quote! { #imp #attrs }
}

fn gen_program_impl(input: &Input) -> TokenStream {
    let ident = &input.ident;
    let attr_ident = &input.attr_ident;
    let data_field = &input.program_data;

    quote! {
        impl ::willow::Program for #ident {
            type AttrStruct = #attr_ident;

            fn create_internally(gl: &::willow::WebGlRenderingContext) -> Self {
                let program = gl.create_program().expect("Cannot initialize program");
                let vertex_shader = gl.create_shader(::willow::WebGlRenderingContext::VERTEX_SHADER).expect("Cannot initialize vertex shader");
                let fragment_shader = gl.create_shader(::willow::WebGlRenderingContext::FRAGMENT_SHADER).expect("Cannot initialize fragment shader");
                Self {
                    #data_field: ProgramData {
                        program,
                        vertex_shader,
                        fragment_shader,
                    },
                }
            }
        }
    }
}

fn gen_attrs(input: &Input) -> TokenStream {
    let vis = &input.vis;
    let attr_ident = &input.attr_ident;

    quote! {
        #vis struct #attr_ident {}

        impl ::willow::ProgramAttrStruct for #attr_ident {
            fn fields_count() -> usize {
                todo!()
            }

            fn field_offset(i: usize) -> usize {
                todo!()
            }
        }
    }
}
