use proc_macro2::{Span, TokenStream};
use quote::{quote_spanned, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;

pub struct Input {
    pub vertex_source: CodeSource,
    pub fragment_source: CodeSource,

    pub attributes: Vec<Attribute>,
    pub uniforms: Vec<Uniform>,
    pub program_data: syn::Ident,

    pub vis: syn::Visibility,
    pub ident: syn::Ident,
    pub attr_ident: syn::Ident,
    pub builder_ident: syn::Ident,
}

pub fn parse_input(ts: TokenStream) -> syn::Result<Input> {
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
            let content;
            syn::parenthesized!(content in input);
            let kw: syn::Ident = content.parse()?;
            Ok(match kw.to_string().as_str() {
                "path" => {
                    let _: syn::Token![=] = content.parse()?;
                    let path: syn::LitStr = content.parse()?;
                    let path = path.value();
                    Self::Path(path)
                }
                "vert" => {
                    let _: syn::Token![=] = content.parse()?;
                    let expr: syn::Expr = content.parse()?;
                    Self::VertexCode(expr)
                }
                "frag" => {
                    let _: syn::Token![=] = content.parse()?;
                    let expr: syn::Expr = content.parse()?;
                    Self::FragmentCode(expr)
                }
                kw => return Err(content.error(format!("Unsupported attribute #[willow({})]", kw))),
            })
        }
    }

    let input: syn::DeriveInput = syn::parse2(ts)?;

    let vis = &input.vis;
    let mut vertex_source = None;
    let mut fragment_source = None;

    let input_ident = &input.ident;

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

    let mut program_data = None;

    for field in &fields.named {
        match FieldOutput::from_field(field)? {
            FieldOutput::Attribute(attr) => attributes.push(attr),
            FieldOutput::Uniform(unif) => uniforms.push(unif),
            FieldOutput::ProgramData(ident) => program_data = Some(ident),
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
    let program_data = match program_data {
        Some(data) => data,
        None => return Err(syn::Error::new_spanned(
            &input.fields,
            "Program struct must have exactly one field with type ProgramData or #[willow(data)]",
        )),
    };

    Ok(Input {
        vertex_source,
        fragment_source,
        attributes,
        uniforms,
        program_data,
        vis: vis.clone(),
        ident: input_ident.clone(),
        attr_ident: quote::format_ident!("{}Attr", &input_ident),
        builder_ident: quote::format_ident!("{}Draw", &input_ident),
    })
}

pub enum CodeSource {
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

pub enum FieldOutput {
    Attribute(Attribute),
    Uniform(Uniform),
    ProgramData(syn::Ident),
}

fn is_ending_ident(path: &syn::Path, name: &str) -> bool {
    if let Some(segment) = path.segments.last() {
        segment.ident == name
    } else {
        false
    }
}

impl FieldOutput {
    fn from_field(field: &syn::Field) -> syn::Result<Self> {
        enum FieldType {
            Attribute(Box<syn::Type>),
            Uniform(Box<syn::Type>),
            Data,
        }
        let mut field_type = None;

        let field_name = field.ident.as_ref().expect("Fields checked as named");
        let mut gl_name = field_name.to_string();
        let mut normalized = false;

        for attr in &field.attrs {
            if attr.path.is_ident("willow") {
                match syn::parse2::<FieldAttr>(attr.tokens.clone())? {
                    FieldAttr::Attribute(ty) => field_type = Some(FieldType::Attribute(ty)),
                    FieldAttr::Uniform(ty) => field_type = Some(FieldType::Uniform(ty)),
                    FieldAttr::GlName(name) => gl_name = name,
                    FieldAttr::Data => field_type = Some(FieldType::Data),
                    FieldAttr::Normalized => normalized = true,
                }
            }
        }

        if field_type.is_none() {
            match &field.ty {
                syn::Type::Path(path) if path.path.is_ident("ProgramData") => {
                    field_type = Some(FieldType::Data)
                }
                syn::Type::Path(path)
                    if is_ending_ident(&path.path, "Uniform")
                        || is_ending_ident(&path.path, "Attribute") =>
                {
                    let segment = path.path.segments.last().expect("Paths must be nonempty");
                    let ty = match &segment.arguments {
                        syn::PathArguments::AngleBracketed(args) => {
                            let arg = args
                                .args
                                .first()
                                .expect("AngleBracketed arguments must be nonempty");
                            match arg {
                                syn::GenericArgument::Type(ty) => Box::new(ty.clone()),
                                arg => {
                                    return Err(syn::Error::new_spanned(
                                        arg,
                                        "Attribute/Uniform provided with non-type argument",
                                    ))
                                }
                            }
                        }
                        _ => {
                            return Err(syn::Error::new_spanned(
                                segment,
                                "Attribute/Uniform requires a type parameter",
                            ))
                        }
                    };
                    if is_ending_ident(&path.path, "Attribute") {
                        field_type = Some(FieldType::Attribute(ty));
                    } else if is_ending_ident(&path.path, "Uniform") {
                        field_type = Some(FieldType::Uniform(ty));
                    } else {
                        unreachable!()
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
            FieldType::Attribute(ty) => FieldOutput::Attribute(Attribute {
                field: field_name.clone(),
                ty,
                gl: gl_name,
                normalized,
            }),
            FieldType::Uniform(ty) => FieldOutput::Uniform(Uniform {
                field: field_name.clone(),
                gl: gl_name,
                ty,
            }),
            FieldType::Data => FieldOutput::ProgramData(field_name.clone()),
        })
    }
}

pub struct Attribute {
    pub field: syn::Ident,
    pub ty: Box<syn::Type>,
    pub gl: String,
    pub normalized: bool,
}

pub struct Uniform {
    pub field: syn::Ident,
    pub gl: String,
    pub ty: Box<syn::Type>,
}

enum FieldAttr {
    Attribute(Box<syn::Type>),
    Uniform(Box<syn::Type>),
    GlName(String),
    Data,
    Normalized,
}

impl Parse for FieldAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        syn::parenthesized!(content in input);
        let kw: syn::Ident = content.parse()?;
        Ok(match kw.to_string().as_str() {
            "normalized" => Self::Normalized,
            "attribute" => {
                let inner;
                syn::parenthesized!(inner in content);
                let ty: syn::Type = inner.parse()?;
                Self::Attribute(Box::new(ty))
            }
            "uniform" => {
                let inner;
                syn::parenthesized!(inner in content);
                let ty: syn::Type = inner.parse()?;
                Self::Uniform(Box::new(ty))
            }
            "gl_name" => {
                let _: syn::Token![=] = content.parse()?;
                let str: syn::LitStr = content.parse()?;
                Self::GlName(str.value())
            }
            "data" => Self::Data,
            kw => return Err(content.error(format!("Unsupported attribute #[willow({})]", kw))),
        })
    }
}
