use proc_macro2::TokenStream;
use quote::{
    quote,
    ToTokens,
};
use syn::{
    parse::{
        Parse,
        ParseStream,
    },
    punctuated::Punctuated,
    spanned::Spanned,
    Error,
    ExprCall,
    Field,
    Fields,
    GenericParam,
    Ident,
    ItemStruct,
    Lit,
    LitStr,
    MetaNameValue,
    Result,
    Token,
};

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
                        offset = Some(value.base10_parse::<u64>()?.to_token_stream())
                    }
                    Lit::Str(value) => offset = Some(value.parse::<ExprCall>()?.to_token_stream()),
                    _ => return Err(Error::new(kv.lit.span(), "expected an interger")),
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
    size: usize,
}

impl Parse for StructArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let input_span = input.span();
        let vars = [MetaNameValue::parse(input)?];
        // let vars: Punctuated<MetaNameValue, syn::token::Comma> =
        //     Punctuated::<MetaNameValue, Token![,]>::parse_terminated(input)?;

        let mut size = None;

        for kv in &vars {
            if kv.path.is_ident("size") {
                let Lit::Int(value) = &kv.lit else {
                    return Err(Error::new(kv.lit.span(), "expected an interger"));
                };

                size = Some(value.base10_parse()?);
            } else {
                return Err(Error::new(kv.path.span(), "unknown attribute"));
            }
        }

        Ok(Self {
            size: size.ok_or(Error::new(input_span, "missing size = \"...\" attribute"))?,
        })
    }
}

fn extract_struct_fields(fields: &Fields) -> Result<Vec<(FieldArgs, Field)>> {
    match fields {
        Fields::Named(fields) => {
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
        _ => Err(Error::new(fields.span(), "expected only named fields")),
    }
}

fn generate_reference_accessors(
    obj_name: &str,
    fields: &[(FieldArgs, Field)],
) -> Result<TokenStream> {
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
        let name_str = format!("{}", name);

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

        result.push(quote! {
            #(#attrs)*
            #[must_use]
            fn #name (&self) -> Result<#ty, raw_struct::AccessError> {
                use raw_struct::{ AccessMode, MemoryViewEx };

                let offset = #offset;
                <#ty>::from_memory(self.object_memory(), offset).map_err(|err| raw_struct::AccessError {
                    object: concat!(module_path!(), "::", #obj_name).into(),
                    member: Some(#name_str),

                    offset,
                    size: core::mem::size_of::<#ty>(),
                    mode: AccessMode::Read,

                    source: err,
                })
            }
        });
    }

    Ok(quote! {
        #(#result)*
    }
    .into_token_stream())
}

pub fn raw_struct(attr: TokenStream, input: TokenStream) -> Result<TokenStream> {
    let args = syn::parse2::<StructArgs>(attr)?;
    let target = syn::parse2::<ItemStruct>(input)?;

    let struct_size = args.size;
    let struct_name = target.ident.clone();
    let struct_name_str = format!("{}", target.ident);

    let struct_attrs = target
        .attrs
        .iter()
        .map(|attr| {
            if attr.path.is_ident("doc") {
                Ok(attr)
            } else {
                Err(Error::new(attr.span(), "struct attributes are supported"))
            }
        })
        .collect::<Result<Vec<_>>>()?;

    let fields = extract_struct_fields(&target.fields)?;
    let accessors = generate_reference_accessors(&struct_name_str, &fields)?;

    let generics = target.generics.clone();
    let ty_list = generics
        .params
        .iter()
        .filter_map(|ty| match ty {
            GenericParam::Type(ty) => Some(ty.ident.clone().into_token_stream()),
            GenericParam::Lifetime(lifetime) => Some(lifetime.lifetime.clone().into_token_stream()),
            GenericParam::Const(_) => None,
        })
        .collect::<Vec<_>>();
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let impl_name = Ident::new(
        &format!("{}Implementation", struct_name),
        struct_name.span(),
    );

    let impl_ty_generics = {
        let mut generics = generics.clone();
        generics.params.insert(
            0,
            syn::parse_quote! {
                MemoryViewT: raw_struct::MemoryView + 'static
            },
        );
        generics
    };
    let (impl_impl_generics, impl_ty_generics, impl_where_clause) =
        impl_ty_generics.split_for_impl();

    let struct_vis = target.vis;
    Ok(quote! {
        #(#struct_attrs)*
        #struct_vis trait #struct_name #ty_generics : raw_struct::ViewableBase #where_clause {
            #accessors
        }


        #struct_vis struct #impl_name #impl_ty_generics (MemoryViewT, core::marker::PhantomData<(#(#ty_list,)*)>) #impl_where_clause;
        impl #impl_impl_generics #struct_name #ty_generics for #impl_name #impl_ty_generics #impl_where_clause {}
        impl #impl_impl_generics raw_struct::ViewableBase
            for #impl_name #impl_ty_generics #impl_where_clause
        {
            fn object_memory(&self) -> &dyn raw_struct::MemoryView {
                &self.0
            }
        }
        impl #impl_impl_generics raw_struct::ViewableImplementation<MemoryViewT, dyn #struct_name #ty_generics>
            for #impl_name #impl_ty_generics #impl_where_clause
        {
            fn object_memory(&self) -> &MemoryViewT {
                &self.0
            }

            fn as_trait(&self) -> &(dyn #struct_name #ty_generics + 'static) {
                self
            }
        }


        impl #impl_generics raw_struct::Viewable<dyn #struct_name #ty_generics> for dyn #struct_name #ty_generics + 'static #where_clause {
            type Memory = [u8; #struct_size];
            type Implementation<MemoryViewT: raw_struct::MemoryView + 'static> = #impl_name #impl_ty_generics;

            fn create<M: raw_struct::MemoryView + 'static>(memory: M) -> Self::Implementation<M> {
                #impl_name (memory, Default::default())
            }

            fn name() -> raw_struct::Cow<'static, str> {
                concat!(module_path!(), "::", #struct_name_str).into()
            }
        }

    })
}
