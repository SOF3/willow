use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;

use super::parse::Input;

pub fn gen_code(input: &Input) -> TokenStream {
    let imp = gen_program_impl(input);
    let attrs = gen_attrs(input);
    quote! { #imp #attrs }
}

fn gen_program_impl(input: &Input) -> TokenStream {
    let ident = &input.ident;
    let attr_ident = &input.attr_ident;
    let data_field = &input.program_data;
    let vert_code = &input.vertex_source;
    let frag_code = &input.fragment_source;

    let create_internally = quote! {
        fn create_internally(context: &::willow::Context) -> Self {
            let gl = &context.native;

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
    };

    let compile_shaders = quote! {
        fn compile_shaders(&self, context: &::willow::Context) {
            let gl = &context.native;

            gl.shader_source(&self.#data_field.vertex_shader, #vert_code);
            gl.shader_source(&self.#data_field.fragment_shader, #frag_code);

            gl.compile_shader(&self.#data_field.vertex_shader);
            gl.compile_shader(&self.#data_field.fragment_shader);

            #[cfg(debug_assertions)]
            {
                for shader in &[&self.#data_field.vertex_shader, &self.#data_field.fragment_shader] {
                    let value = gl.get_shader_parameter(shader, ::willow::WebGlRenderingContext::COMPILE_STATUS);
                    if !value.is_truthy() {
                        let log = gl.get_shader_info_log(shader);
                        panic!("Error compiling {}: {}", stringify!(#ident), log.unwrap_or_default());
                    }
                }
            }
        }
    };

    let link_shaders = quote! {
        fn link_shaders(&self, context: &::willow::Context) {
            let gl = &context.native;

            gl.attach_shader(&self.#data_field.program, &self.#data_field.vertex_shader);
            gl.attach_shader(&self.#data_field.program, &self.#data_field.fragment_shader);
            gl.link_program(&self.#data_field.program);

            #[cfg(debug_assertions)]
            {
                let value = gl.get_program_parameter(&self.#data_field.program, ::willow::WebGlRenderingContext::LINK_STATUS);
                if !value.is_truthy() {
                    let log = gl.get_program_info_log(&self.#data_field.program);
                    panic!("Error linking {}: {}", stringify!(#ident), log.unwrap_or_default());
                }
            }
        }
    };

    let attr_fields = input.attributes.iter().map(|attr| &attr.field);
    let attr_names = input.attributes.iter().map(|attr| &attr.gl);
    let field_index = 0..input.attributes.len();
    let apply_attrs = quote! {
        fn apply_attrs(&self, context: &::willow::Context, buffer: &::willow::Buffer<Self::AttrStruct>) {
            let gl = &context.native;

            gl.bind_buffer(::willow::WebGlRenderingContext::ARRAY_BUFFER, Some(&buffer.buf));

            #(
                let location = self.#attr_fields.get_location(context, &self.#data_field.program, #attr_names);
                buffer.bind_to_attr(context, location, #field_index);
            )*
        }
    };

    quote! {
        impl ::willow::Program for #ident {
            type AttrStruct = #attr_ident;

            #create_internally

            #compile_shaders

            #link_shaders

            #apply_attrs
        }
    }
}

fn gen_attrs(input: &Input) -> TokenStream {
    let vis = &input.vis;
    let attr_ident = &input.attr_ident;

    let field_def = input.attributes.iter().map(|attr| {
        let name = &attr.field;
        let ty = &attr.ty;
        quote!(#name: #ty)
    });

    let num_fields = input.attributes.len();
    let fn_fields_count = quote! { fn fields_count() -> usize { #num_fields } };

    fn define_function<T: ToTokens>(
        name: &str,
        return_type: TokenStream,
        values: impl Iterator<Item = T>,
        spans: impl Iterator<Item = Span>,
    ) -> TokenStream {
        let name = syn::Ident::new(name, Span::call_site());

        let arms = values.zip(spans).enumerate().map(|(index, (value, span))| {
            quote_spanned! { span =>
                #index => #value,
            }
        });

        quote! {
            fn #name (i: usize) -> #return_type {
                match i {
                    #(#arms)*
                    _ => panic!("Nonexistent field"),
                }
            }
        }
    }

    let fn_field_gl_name = define_function(
        "field_gl_name",
        quote!(&'static str),
        input.attributes.iter().map(|attr| {
            let name = &attr.gl;
            quote!(#name)
        }),
        input.attributes.iter().map(|attr| attr.ty.span()),
    );

    let fn_field_offset = define_function(
        "field_offset",
        quote!(usize),
        input.attributes.iter().map(|attr| {
            let name = &attr.field;
            let ty = &attr.ty;
            quote!(::willow::offset_of!(#ty => #name).get_byte_offset())
        }),
        input.attributes.iter().map(|attr| attr.ty.span()),
    );

    let fn_field_type = define_function(
        "field_type",
        quote!(u32),
        input.attributes.iter().map(|attr| {
            let ty = &attr.ty;
            quote!(AttributeType::gl_type(#ty))
        }),
        input.attributes.iter().map(|attr| attr.ty.span()),
    );

    let fn_field_num_comps = define_function(
        "field_num_comps",
        quote!(usize),
        input.attributes.iter().map(|attr| {
            let ty = &attr.ty;
            quote!(AttributeType::num_comps(#ty))
        }),
        input.attributes.iter().map(|attr| attr.ty.span()),
    );

    let fn_field_normalized = define_function(
        "field_normalized",
        quote!(bool),
        input.attributes.iter().map(|attr| {
            let normalized = attr.normalized;
            quote!(#normalized)
        }),
        input.attributes.iter().map(|attr| attr.ty.span()),
    );

    quote! {
        #vis struct #attr_ident { #(#field_def),* }

        impl ::willow::AttrStruct for #attr_ident {
            #fn_fields_count

            #fn_field_gl_name
            #fn_field_offset
            #fn_field_type
            #fn_field_num_comps
            #fn_field_normalized
        }
    }
}
