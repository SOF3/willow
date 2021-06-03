use heck::CamelCase;
use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;

use super::parse::Input;

pub fn gen_code(input: &Input) -> TokenStream {
    let imp = gen_program_impl(input);
    let attrs = gen_attrs(input);
    let builder = gen_builder(input);
    // eprintln!("builder: {}", &builder);
    quote! { #imp #attrs #builder }
}

fn gen_program_impl(input: &Input) -> TokenStream {
    let ident = &input.ident;
    let attr_ident = &input.attr_ident;
    let data_field = &input.program_data;
    let vert_code = &input.vertex_source;
    let frag_code = &input.fragment_source;

    let init_attrs = input.attributes.iter().map(|attr| {
        let name = &attr.field;
        let init_expr = quote!(::willow::Attribute::create_from_macro());
        quote!(#name: #init_expr)
    });
    let init_uniforms = input.uniforms.iter().map(|attr| {
        let name = &attr.field;
        let init_expr = quote!(::willow::Uniform::create_from_macro());
        quote!(#name: #init_expr)
    });

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
                #(#init_attrs,)*
                #(#init_uniforms,)*
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
                let location = self.#attr_fields.get_location(context, &self.#data_field, #attr_names);
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
            quote!(::willow::offset_of!(Self => #name).get_byte_offset())
        }),
        input.attributes.iter().map(|attr| attr.ty.span()),
    );

    let fn_field_type = define_function(
        "field_type",
        quote!(u32),
        input.attributes.iter().map(|attr| {
            let ty = &attr.ty;
            quote!(<#ty as ::willow::AttributeType>::gl_type())
        }),
        input.attributes.iter().map(|attr| attr.ty.span()),
    );

    let fn_field_num_comps = define_function(
        "field_num_comps",
        quote!(usize),
        input.attributes.iter().map(|attr| {
            let ty = &attr.ty;
            quote!(<#ty as ::willow::AttributeType>::num_comps())
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

fn gen_builder(input: &Input) -> TokenStream {
    let ident = &input.ident;
    let vis = &input.vis;
    let builder_ident = &input.builder_ident;
    let data_field = &input.program_data;

    let generics: Vec<_> = input
        .uniforms
        .iter()
        .map(|uniform| {
            let raw = format!("has {}", uniform.field.to_string());
            let camel = raw.to_camel_case();
            syn::Ident::new(camel.as_str(), uniform.field.span())
        })
        .collect();

    let field_names: Vec<_> = input
        .uniforms
        .iter()
        .map(|uniform| &uniform.field)
        .collect();
    let types: Vec<_> = input.uniforms.iter().map(|uniform| &uniform.ty).collect();

    let builders = field_names.iter().enumerate().map(|(i, field_name)| {
        let empty_generics = generics.iter().enumerate().map(|(j, ident)| {
            if i == j { quote!(()) } else { quote!(#ident) }
        });
        let filled_generics = generics.iter().enumerate().map(|(j, ident)| {
            if i == j { types[i].to_token_stream() } else { quote!(#ident) }
        });

        let other_fields = input.uniforms.iter().enumerate().filter(|&(j, _)| j != i)
            .map(|(_, uniform)| &uniform.field);

        let other_generics = generics.iter().enumerate().filter(|&(j, _)| j != i).map(|(_, v)| v);

        let ty = &input.uniforms[i].ty;

        quote! {
            impl<'program, #(#other_generics),*> #builder_ident<'program, #(#empty_generics),*> {
                #vis fn #field_name(self, #field_name: #ty) -> #builder_ident<'program, #(#filled_generics),*> {
                    let Self {
                        program,
                        #field_name: (),
                        #(#other_fields),*
                    } = self;

                    #builder_ident {
                        program,
                        #(#field_names),*
                    }
                }
            }
        }
    });

    let gl_names = input.uniforms.iter().map(|uniform| &uniform.gl);

    let draw_def = quote! {
        impl<'program> #builder_ident<'program, #(#types),*> {
            pub fn draw(self, context: &::willow::Context) -> Option<()> {
                #({
                    let location = self.program.#field_names.get_location(context, &self.program.#data_field, #gl_names)?;
                    ::willow::UniformType::apply_uniform(self.#field_names, &context.native, location);
                })*

                Some(())
            }
        }
    };

    quote! {
        #[derive(Clone, Copy)]
        #[must_use = "This is a builder type"]
        #vis struct #builder_ident<'program, #(#generics),*> {
            program: &'program #ident,
            #(#field_names: #generics,)*
        }

        #(#builders)*

        #draw_def
    }
}
