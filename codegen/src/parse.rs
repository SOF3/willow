use proc_macro2::{Span, TokenStream};
use quote::{quote_spanned, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;

pub struct Input {
    vertex_source: CodeSource,
    fragment_source: CodeSource,

    attributes: Vec<Attribute>,
    uniforms: Vec<Uniform>,
}

pub fn parse_input(ts: TokenStream) -> syn::Result<Input> {
    let input: syn::DeriveInput = syn::parse2(ts)?;

    let mut vertex_source = None;
    let mut fragment_source = None;

    for attr in &input.attrs {
        if attr.path.is_ident("willow") {
            match syn::parse2::<StructAttr>(attr.tokens.clone())? {
                StructAttr::Path(path) => {
                    vertex_source = Some(CodeSource::File(attr.span(), format!("{}.vert", &path)));
                    fragment_source =
                        Some(CodeSource::File(attr.span(), format!("{}.frag", &path)));
                }
                StructAttr::VertexCode(expr) => {
                    vertex_source = Some(CodeSource::Expr(Box::new(expr)));
                }
                StructAttr::FragmentCode(expr) => {
                    fragment_source = Some(CodeSource::Expr(Box::new(expr)));
                }
            }
        }
    }

    let input = match &input.data {
        syn::Data::Struct(s) => s,
        _ => {
            return Err(syn::Error::new_spanned(
                &input,
                "willow::Program can only derive from structs",
            ))
        }
    };

    let fields = match &input.fields {
        syn::Fields::Named(fields) => fields,
        fields => {
            return Err(syn::Error::new_spanned(
                fields,
                "willow::Program can only derive from named structs",
            ))
        }
    };

    let mut attributes = Vec::new();
    let mut uniforms = Vec::new();

    for field in &fields.named {
        match FieldOutput::from_field(field)? {
            FieldOutput::Attribute(attr) => attributes.push(attr),
            FieldOutput::Uniform(unif) => uniforms.push(unif),
        }
    }

    let vertex_source = match vertex_source {
        Some(s) => s,
        None => {
            return Err(syn::Error::new_spanned(
                &input.fields,
                "Cannot infer vertex code",
            ))
        }
    };
    let fragment_source = match fragment_source {
        Some(s) => s,
        None => {
            return Err(syn::Error::new_spanned(
                &input.fields,
                "Cannot infer fragment code",
            ))
        }
    };

    Ok(Input {
        vertex_source,
        fragment_source,
        attributes,
        uniforms,
    })
}

enum FieldOutput {
    Attribute(Attribute),
    Uniform(Uniform),
}

struct Attribute {
    field: String,
    gl: String,
}

struct Uniform {
    field: String,
    gl: String,
    ty: Box<syn::Type>,
}

impl FieldOutput {
    fn from_field(field: &syn::Field) -> syn::Result<Self> {
        enum FieldType {
            Attribute,
            Uniform(Box<syn::Type>),
        }
        let mut field_type = None;

        let field_name = field
            .ident
            .as_ref()
            .expect("Fields checked as named")
            .to_string();
        let mut gl_name = field_name.clone();

        for attr in &field.attrs {
            if attr.path.is_ident("willow") {
                match syn::parse2::<FieldAttr>(attr.tokens.clone())? {
                    FieldAttr::Attribute => field_type = Some(FieldType::Attribute),
                    FieldAttr::Uniform(ty) => field_type = Some(FieldType::Uniform(ty)),
                    FieldAttr::GlName(name) => gl_name = name,
                }
            }
        }

        if field_type.is_none() {
            match &field.ty {
                syn::Type::Path(path) if path.path.is_ident("Attribute") => {
                    field_type = Some(FieldType::Attribute)
                }
                syn::Type::Path(path) if path.path.is_ident("Uniform") => {
                    let segment = path.path.segments.last().expect("Paths must be nonempty");
                    match &segment.arguments {
                        syn::PathArguments::AngleBracketed(args) => {
                            let arg = args
                                .args
                                .first()
                                .expect("AngleBracketed arguments must be nonempty");
                            match arg {
                                syn::GenericArgument::Type(ty) => {
                                    field_type = Some(FieldType::Uniform(Box::new(ty.clone())))
                                }
                                arg => {
                                    return Err(syn::Error::new_spanned(
                                        arg,
                                        "Uniform provided with non-type argument",
                                    ))
                                }
                            }
                        }
                        _ => {
                            return Err(syn::Error::new_spanned(
                                segment,
                                "Uniform requires a type parameter",
                            ))
                        }
                    }
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        &field.ty,
                        "This field appears to be irrelevant with the GLSL",
                    ))
                }
            }
        }

        Ok(match field_type.expect("checked") {
            FieldType::Attribute => FieldOutput::Attribute(Attribute {
                field: field_name,
                gl: gl_name,
            }),
            FieldType::Uniform(ty) => FieldOutput::Uniform(Uniform {
                field: field_name,
                gl: gl_name,
                ty,
            }),
        })
    }
}

enum StructAttr {
    /// Specifies the source GLSL files
    Path(String),
    /// Specifies the vertex GLSL code dynamically
    VertexCode(syn::Expr),
    /// Specifies the fragment GLSL code dynamically
    FragmentCode(syn::Expr),
}

impl Parse for StructAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let kw: syn::Ident = input.parse()?;
        Ok(match kw.to_string().as_str() {
            "path" => {
                let _: syn::Token![=] = input.parse()?;
                let path: syn::LitStr = input.parse()?;
                let path = path.value();
                Self::Path(path)
            }
            "vert" => {
                let _: syn::Token![=] = input.parse()?;
                let expr: syn::Expr = input.parse()?;
                Self::VertexCode(expr)
            }
            "frag" => {
                let _: syn::Token![=] = input.parse()?;
                let expr: syn::Expr = input.parse()?;
                Self::FragmentCode(expr)
            }
            kw => return Err(input.error(format!("Unsupported attribute #[willow({})]", kw))),
        })
    }
}

enum FieldAttr {
    Attribute,
    Uniform(Box<syn::Type>),
    GlName(String),
}

impl Parse for FieldAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let kw: syn::Ident = input.parse()?;
        Ok(match kw.to_string().as_str() {
            "attribute" => Self::Attribute,
            "uniform" => {
                let content;
                syn::parenthesized!(content in input);
                let ty: syn::Type = content.parse()?;
                Self::Uniform(Box::new(ty))
            }
            "gl_name" => {
                let _: syn::Token![=] = input.parse()?;
                let str: syn::LitStr = input.parse()?;
                Self::GlName(str.value())
            }
            kw => return Err(input.error(format!("Unsupported attribute #[willow({})]", kw))),
        })
    }
}

enum CodeSource {
    File(Span, String),
    Expr(Box<syn::Expr>),
}

impl ToTokens for CodeSource {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::File(span, path) => {
                tokens.extend(quote_spanned! { *span=> include_str!(#path) });
            }
            Self::Expr(expr) => {
                expr.to_tokens(&mut *tokens);
            }
        };
    }
}
