use proc_macro2::TokenStream;
use quote::{
    quote,
    ToTokens,
};
use syn::{
    parse::{
        discouraged::Speculative,
        Parse,
        ParseStream,
    },
    punctuated::Punctuated,
    spanned::Spanned,
    Error,
    Expr,
    Field,
    Fields,
    GenericParam,
    ItemStruct,
    Lit,
    MetaNameValue,
    Path,
    Result,
    Token,
};

#[derive(Debug)]
struct FieldArgs {
    // field(offset = 0x00)
    offset: TokenStream,
}

impl Parse for FieldArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let fork = input.fork();
        let Ok(vars) = fork.call(Punctuated::<MetaNameValue, Token![,]>::parse_terminated) else {
            /* the input is already the offset value */
            return Ok(Self {
                offset: input.parse()?,
            });
        };
        input.advance_to(&fork);

        let mut offset = None;

        for kv in &vars {
            if kv.path.is_ident("offset") {
                match &kv.lit {
                    Lit::Int(value) => {
                        offset = Some(value.base10_parse::<usize>()?.to_token_stream())
                    }
                    Lit::Str(value) => offset = Some(value.parse::<Expr>()?.to_token_stream()),
                    _ => return Err(Error::new(kv.lit.span(), "expected an interger or string")),
                }
            } else {
                return Err(Error::new(kv.path.span(), "unknown attribute"));
            }
        }

        Ok(Self {
            offset: offset.ok_or(Error::new(vars.span(), "missing offset = \"...\""))?,
        })
    }
}

#[derive(Debug)]
struct StructArgs {
    memory: Option<TokenStream>,
    inherits: Option<Path>,
    resolver: Path,
}

impl Parse for StructArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let vars: Punctuated<MetaNameValue, syn::token::Comma> =
            Punctuated::<MetaNameValue, Token![,]>::parse_terminated(input)?;

        let mut size = None;
        let mut memory = None;
        let mut inherits = None;
        let mut resolver = None;

        for kv in &vars {
            if kv.path.is_ident("size") {
                match &kv.lit {
                    Lit::Int(value) => {
                        size = Some(value.base10_parse::<usize>()?.to_token_stream())
                    }
                    Lit::Str(value) => size = Some(value.parse::<Expr>()?.to_token_stream()),
                    _ => return Err(Error::new(kv.lit.span(), "expected an interger or string")),
                }
            } else if kv.path.is_ident("memory") {
                match &kv.lit {
                    Lit::Str(value) => memory = Some(value.parse::<Expr>()?.to_token_stream()),
                    _ => return Err(Error::new(kv.lit.span(), "expected a string")),
                }
            } else if kv.path.is_ident("inherits") {
                match &kv.lit {
                    Lit::Str(value) => inherits = Some(value.parse::<Path>()?),
                    _ => return Err(Error::new(kv.lit.span(), "expected a string")),
                }
            } else if kv.path.is_ident("resolver") {
                match &kv.lit {
                    Lit::Str(value) => resolver = Some(value.parse::<Path>()?),
                    _ => return Err(Error::new(kv.lit.span(), "expected a string")),
                }
            } else {
                return Err(Error::new(kv.path.span(), "unknown attribute"));
            }
        }

        let size = size.map(|size: TokenStream| quote::quote!([u8; #size]).to_token_stream());
        Ok(Self {
            memory: memory.or(size),
            inherits,
            resolver: resolver.unwrap_or_else(|| syn::parse_quote! { ::core::convert::identity }),
        })
    }
}

fn extract_struct_fields(fields: &Fields) -> Result<Vec<(FieldArgs, Field)>> {
    let Fields::Named(fields) = fields else {
        return Err(Error::new(fields.span(), "only named fields supported"));
    };

    let mut result = Vec::with_capacity(fields.named.len());
    for field in fields.named.iter() {
        let mut field = field.clone();
        let attr_index = field
            .attrs
            .iter()
            .position(|attr| attr.path.is_ident("field"))
            .ok_or_else(|| {
                Error::new(
                    field.span(),
                    "every field has to be attributed with #[field(...)]",
                )
            })?;

        let attr = field.attrs.remove(attr_index).parse_args::<FieldArgs>()?;
        result.push((attr, field));
    }
    Ok(result)
}

fn generate_field_constants(resolver: &Path, fields: &[(FieldArgs, Field)]) -> Result<TokenStream> {
    let mut result = Vec::<TokenStream>::with_capacity(fields.len() * 2);

    for (field_args, field) in fields.iter() {
        let ty = &field.ty;
        let Some(ident) = &field.ident else {
            continue;
        };
        let ident_str = format!("{ident}");

        let offset = &field_args.offset;
        let attrs = field
            .attrs
            .iter()
            .map(|attr| {
                if attr.path.is_ident("doc") {
                    Ok(attr)
                } else {
                    Err(Error::new(attr.span(), "attributes are not supported"))
                }
            })
            .collect::<Result<Vec<_>>>()?;

        let vis = &field.vis;
        result.push(quote! {
            #(#attrs)*
            #[allow(non_upper_case_globals)]
            #vis const #ident: &::raw_struct::TypedViewableField<Self, #ty> = &::raw_struct::TypedViewableField::define(#ident_str, &|| {
                #resolver(#offset) as u64
            });
        });
    }

    Ok(quote! {
        #(#result)*
    }
    .into_token_stream())
}

fn generate_struct_definition(args: &StructArgs, target: &ItemStruct) -> Result<TokenStream> {
    let attr_clone_copy = syn::parse_quote! {
        #[derive(Clone, Copy)]
    };

    let mut attributes = target.attrs.iter().collect::<Vec<_>>();
    attributes.push(&attr_clone_copy);

    let vis = &target.vis;
    let name = &target.ident;
    let generics = &target.generics;

    let type_list = target
        .generics
        .params
        .iter()
        .filter_map(|ty| match ty {
            GenericParam::Type(ty) => Some(ty.ident.clone().into_token_stream()),
            GenericParam::Lifetime(lifetime) => Some(lifetime.lifetime.clone().into_token_stream()),
            GenericParam::Const(_) => None,
        })
        .collect::<Vec<_>>();

    let inherits = if let Some(inherits) = &args.inherits {
        Some(quote! { impl ::raw_struct::ViewableExtends< #inherits > for #name {} })
    } else {
        None
    };

    Ok(quote! {
        #(#attributes)*
        #vis struct #name #generics {
            _generics: core::marker::PhantomData<(#(#type_list,)*)>,
        }

        #inherits
    })
}

pub fn raw_struct(attr: TokenStream, input: TokenStream) -> Result<TokenStream> {
    let args = syn::parse2::<StructArgs>(attr)?;
    let target = syn::parse2::<ItemStruct>(input)?;

    let struct_name = target.ident.clone();
    let struct_name_str = format!("{}", target.ident);

    let fields = extract_struct_fields(&target.fields)?;
    let field_constants = generate_field_constants(&args.resolver, &fields)?;

    let (impl_generics, ty_generics, where_clause) = target.generics.split_for_impl();

    let struct_def = self::generate_struct_definition(&args, &target)?;

    let sized_impl = args.memory.map(|memory| quote! {
        impl #impl_generics ::raw_struct::ViewableSized for #struct_name #ty_generics #where_clause {
            type Memory = #memory;
        }
    });

    Ok(quote! {
        #struct_def

        impl #impl_generics #struct_name #ty_generics #where_clause {
            #field_constants
        }

        impl #impl_generics ::raw_struct::Viewable for #struct_name #ty_generics #where_clause {
            fn name() -> &'static str {
                #struct_name_str
            }
        }

        #sized_impl
    })
}
