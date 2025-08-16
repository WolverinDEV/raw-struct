use proc_macro2::{
    Span,
    TokenStream,
};
use quote::{
    quote,
    ToTokens,
};
use syn::{
    parse::{
        Parse,
        ParseStream,
    },
    parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
    Error,
    Expr,
    Field,
    Fields,
    GenericArgument,
    GenericParam,
    Ident,
    ItemStruct,
    Lit,
    LitStr,
    MetaNameValue,
    Path,
    PathArguments,
    Result,
    Token,
};

struct StaticIdent(&'static str);

impl ToTokens for StaticIdent {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        Ident::new(self.0, Span::call_site()).to_tokens(tokens);
    }
}

const IDENT_MEMORY_VIEW_T: StaticIdent = StaticIdent("_MemoryViewT");

#[derive(Debug)]
struct FieldArgs {
    // field(offset = 0x00, getter = "", setter = "")
    offset: TokenStream,

    getter: Option<Ident>,
    _setter: Option<LitStr>,
}

impl Parse for FieldArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let vars: Punctuated<MetaNameValue, syn::token::Comma> =
            Punctuated::<MetaNameValue, Token![,]>::parse_terminated(input)?;

        let mut offset = None;

        let mut getter = None;
        let mut setter = None;

        for kv in &vars {
            if kv.path.is_ident("offset") {
                match &kv.lit {
                    Lit::Int(value) => {
                        offset = Some(value.base10_parse::<usize>()?.to_token_stream())
                    }
                    Lit::Str(value) => offset = Some(value.parse::<Expr>()?.to_token_stream()),
                    _ => return Err(Error::new(kv.lit.span(), "expected an interger or string")),
                }
            } else if kv.path.is_ident("getter") {
                let Lit::Str(value) = &kv.lit else {
                    return Err(Error::new(kv.lit.span(), "expected a string"));
                };

                getter = Some(value.parse()?);
            } else if kv.path.is_ident("setter") {
                let Lit::Str(value) = &kv.lit else {
                    return Err(Error::new(kv.lit.span(), "expected a string"));
                };

                setter = Some(value.parse()?);
            } else {
                return Err(Error::new(kv.path.span(), "unknown attribute"));
            }
        }

        Ok(Self {
            offset: offset.ok_or(Error::new(vars.span(), "missing offset = \"...\""))?,

            getter,
            _setter: setter,
        })
    }
}

#[derive(Debug)]
struct StructArgs {
    memory: Option<TokenStream>,
    inherits: Option<Path>,
}

impl Parse for StructArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let vars: Punctuated<MetaNameValue, syn::token::Comma> =
            Punctuated::<MetaNameValue, Token![,]>::parse_terminated(input)?;

        let mut size = None;
        let mut memory = None;
        let mut inherits = None;

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
            } else {
                return Err(Error::new(kv.path.span(), "unknown attribute"));
            }
        }

        let size = size.map(|size: TokenStream| quote::quote!([u8; #size]).to_token_stream());
        Ok(Self {
            memory: memory.or(size),
            inherits,
        })
    }
}

fn add_type_param(path: &mut Path, ty: syn::Type) {
    if let Some(last) = path.segments.last_mut() {
        match &mut last.arguments {
            PathArguments::None => {
                // Turn `None` into `<T>`
                last.arguments =
                    PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                        colon2_token: None,
                        lt_token: Default::default(),
                        args: vec![GenericArgument::Type(ty)].into_iter().collect(),
                        gt_token: Default::default(),
                    });
            }
            PathArguments::AngleBracketed(args) => {
                // Already has <...>, just push
                args.args.push(GenericArgument::Type(ty));
            }
            PathArguments::Parenthesized(_) => {
                // Function-like path segment, can't add normal type params
                panic!("cannot add type arguments to a function-like path segment");
            }
        }
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

fn generate_reference_accessors(fields: &[(FieldArgs, Field)]) -> Result<TokenStream> {
    let mut result = Vec::<TokenStream>::with_capacity(fields.len() * 2);

    for (field_args, field) in fields.iter() {
        let ty = &field.ty;
        let name = if let Some(name) = &field_args.getter {
            name
        } else if let Some(ident) = &field.ident {
            ident
        } else {
            continue;
        };

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

        let vis = field.vis;
        result.push(quote! {
            #(#attrs)*
            #[must_use]
            #vis fn #name (&self) -> Result<#ty, raw_struct::MemoryDecodeError<#IDENT_MEMORY_VIEW_T::AccessError, <#ty as raw_struct::FromMemoryView>::DecodeError>> {
                use raw_struct::{ ViewableImplementation, FromMemoryView };

                let offset = (#offset) as u64;
                <#ty as FromMemoryView>::read_object(self.memory_view(), offset)
            }
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

    let struct_generics = {
        let mut generics = target.generics.clone();
        generics.params.push(syn::parse_quote! {
            #IDENT_MEMORY_VIEW_T = ()
        });
        generics
    };

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

    let memory = if let Some(inherits) = &args.inherits {
        let mut inherits = inherits.clone();
        self::add_type_param(&mut inherits, parse_quote!(#IDENT_MEMORY_VIEW_T));
        quote! { inner: #inherits }
    } else {
        quote! { memory: #IDENT_MEMORY_VIEW_T }
    };

    Ok(quote! {
        #(#attributes)*
        #vis struct #name #struct_generics {
            #memory,
            _type: std::marker::PhantomData<(#(#type_list,)*)>,
        }
    })
}

pub fn raw_struct(attr: TokenStream, input: TokenStream) -> Result<TokenStream> {
    let args = syn::parse2::<StructArgs>(attr)?;
    let target = syn::parse2::<ItemStruct>(input)?;

    let struct_name = target.ident.clone();
    let struct_name_str = format!("{}", target.ident);

    let fields = extract_struct_fields(&target.fields)?;
    let accessors = generate_reference_accessors(&fields)?;

    let (vanilla_impl_generics, vanilla_ty_generics, vanilla_where_clause) =
        target.generics.split_for_impl();

    let impl_generics = {
        let mut generics = target.generics.clone();
        generics.params.push(syn::parse_quote! {
            #IDENT_MEMORY_VIEW_T: raw_struct::MemoryView
        });
        generics
    };
    let (impl_generics, impl_ty_generics, impl_where_clause) = impl_generics.split_for_impl();

    let struct_def = self::generate_struct_definition(&args, &target)?;

    let impl_impl = if let Some(_inherits) = &args.inherits {
        quote! {
            impl #impl_generics raw_struct::ViewableImplementation<#IDENT_MEMORY_VIEW_T> for #struct_name #impl_ty_generics #impl_where_clause {
                fn memory_view(&self) -> &#IDENT_MEMORY_VIEW_T {
                    self.inner.memory_view()
                }

                fn into_memory_view(self) -> #IDENT_MEMORY_VIEW_T {
                    self.inner.into_memory_view()
                }
            }
        }
    } else {
        quote! {
            impl #impl_generics raw_struct::ViewableImplementation<#IDENT_MEMORY_VIEW_T> for #struct_name #impl_ty_generics #impl_where_clause {
                fn memory_view(&self) -> &#IDENT_MEMORY_VIEW_T {
                    &self.memory
                }

                fn into_memory_view(self) -> #IDENT_MEMORY_VIEW_T {
                    self.memory
                }
            }
        }
    };

    let impl_construct_from_memory = if let Some(inherits) = &args.inherits {
        quote! { #struct_name { inner: #inherits ::from_memory(memory), _type: Default::default() } }
    } else {
        quote! { #struct_name { memory, _type: Default::default() } }
    };

    let deref_impl = if let Some(inherits) = &args.inherits {
        let mut inherits = inherits.clone();
        self::add_type_param(&mut inherits, parse_quote!(#IDENT_MEMORY_VIEW_T));
        Some(quote! {
            impl #impl_generics core::ops::Deref for #struct_name #impl_ty_generics #impl_where_clause {
                type Target = #inherits;

                fn deref(&self) -> &Self::Target {
                    &self.inner
                }
            }
        })
    } else {
        None
    };

    let sized_impl = args.memory.map(|memory| quote! {
        impl #vanilla_impl_generics raw_struct::SizedViewable for #struct_name #vanilla_ty_generics #vanilla_where_clause {
            type Memory = #memory;
        }
    });

    Ok(quote! {
        #struct_def

        impl #impl_generics #struct_name #impl_ty_generics #impl_where_clause {
            #accessors
        }

        #impl_impl

        impl #vanilla_impl_generics raw_struct::Viewable for #struct_name #vanilla_ty_generics #vanilla_where_clause {
            type Implementation<#IDENT_MEMORY_VIEW_T: raw_struct::MemoryView> = #struct_name #impl_ty_generics;

            fn name() -> &'static str {
                #struct_name_str
            }

            fn from_memory<#IDENT_MEMORY_VIEW_T: raw_struct::MemoryView>(memory: #IDENT_MEMORY_VIEW_T) -> Self::Implementation<#IDENT_MEMORY_VIEW_T> {
                #impl_construct_from_memory
            }
        }

        #sized_impl

        #deref_impl
    })
}
