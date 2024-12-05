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
    Attribute,
    Error,
    Expr,
    Field,
    Fields,
    Generics,
    Ident,
    ImplGenerics,
    ItemStruct,
    Lit,
    LitStr,
    MetaNameValue,
    Result,
    Token,
    TypeGenerics,
    WhereClause,
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
    memory: TokenStream,
}

impl Parse for StructArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let input_span = input.span();
        let vars: Punctuated<MetaNameValue, syn::token::Comma> =
            Punctuated::<MetaNameValue, Token![,]>::parse_terminated(input)?;

        let mut size = None;
        let mut memory = None;

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
            } else {
                return Err(Error::new(kv.path.span(), "unknown attribute"));
            }
        }

        let size = size.map(|size: TokenStream| quote::quote!([u8; #size]).to_token_stream());
        Ok(Self {
            memory: memory
                .or(size)
                .ok_or(Error::new(input_span, "missing size = \"...\" attribute"))?,
        })
    }
}

struct ViewableField<'a> {
    field: &'a Field,

    attributes: Vec<&'a Attribute>,
    args: FieldArgs,
}

impl<'a> ViewableField<'a> {
    pub fn parse_input(field: &'a Field) -> Result<Self> {
        let mut args = None;
        let mut attributes = Vec::with_capacity(field.attrs.len() - 1);
        for attribute in &field.attrs {
            if attribute.path.is_ident("field") {
                if args.is_some() {
                    return Err(Error::new(
                        attribute.span(),
                        "duplicate #[field(...)] attribute",
                    ));
                }

                args = Some(attribute.parse_args::<FieldArgs>()?);
            } else if attribute.path.is_ident("doc") {
                attributes.push(attribute);
            } else {
                return Err(Error::new(attribute.span(), "attributes are not supported"));
            }
        }

        Ok(Self {
            field,

            attributes,
            args: args.ok_or_else(|| {
                Error::new(
                    field.span(),
                    "every field has to be attributed with #[field(...)]",
                )
            })?,
        })
    }

    pub fn generate_accessor(&self, container: &ViewableGenerator) -> Option<TokenStream> {
        let ty = &self.field.ty;
        let name = if let Some(name) = &self.args.getter {
            name
        } else if let Some(ident) = &self.field.ident {
            ident
        } else {
            return None;
        };
        let name_str = format!("{}", name);

        let offset = &self.args.offset;
        let attrs = &self.attributes;

        let obj_name = &format!("{}", container.source.ident);
        Some(quote! {
            #(#attrs)*
            #[must_use]
            fn #name (&self) -> Result<#ty, raw_struct::AccessError<MemoryView::Error>> {
                use raw_struct::{ AccessMode, FromMemoryView };

                let offset = (#offset) as u64;
                let memory = self.object_memory();
                <#ty as FromMemoryView>::read_object(memory, offset)
                    .map_err(|source| raw_struct::AccessError {
                        object: concat!(module_path!(), "::", #obj_name).into(),
                        member: Some(#name_str .into()),

                        offset,
                        size: core::mem::size_of::<#ty>(),
                        mode: AccessMode::Read,

                        source,
                    })
            }
        })
    }
}

struct ViewableGenerator<'a> {
    source: &'a ItemStruct,
    args: StructArgs,

    attributes: Vec<&'a Attribute>,
    fields: Vec<ViewableField<'a>>,

    generics_impl: ImplGenerics<'a>,
    generics_type: TypeGenerics<'a>,
    generics_where: Option<&'a WhereClause>,

    accessor_name: Ident,
    accessor_generics: Generics,

    instance_name: Ident,
}

impl<'a> ViewableGenerator<'a> {
    fn parse_fields(target: &'a ItemStruct) -> Result<Vec<ViewableField<'a>>> {
        match &target.fields {
            Fields::Named(fields) => fields
                .named
                .iter()
                .map(ViewableField::parse_input)
                .collect::<Result<Vec<_>>>(),
            _ => Err(Error::new(
                target.fields.span(),
                "only named fields are allowed",
            )),
        }
    }

    fn parse_attributes(target: &'a ItemStruct) -> Result<Vec<&'a Attribute>> {
        target
            .attrs
            .iter()
            .map(|attr| {
                if attr.path.is_ident("doc") {
                    Ok(attr)
                } else {
                    Err(Error::new(
                        attr.span(),
                        "only \"doc\" attributes are supported on raw_structs",
                    ))
                }
            })
            .collect::<Result<Vec<_>>>()
    }

    fn new(target: &'a ItemStruct, target_args: StructArgs) -> Result<Self> {
        let fields = Self::parse_fields(&target)?;
        let attributes = Self::parse_attributes(&target)?;

        let (generics_impl, generics_type, generics_where) = target.generics.split_for_impl();

        let accessor_name = Ident::new(&format!("{}_Accessor", &target.ident), target.ident.span());
        let accessor_generics = {
            let mut generics = target.generics.clone();
            generics.params.insert(0, syn::parse_quote! { MemoryView });
            generics.where_clause = if let Some(clause) = generics.where_clause {
                syn::parse_quote! {
                    #clause
                    MemoryView: ::raw_struct::MemoryView + 'static
                }
            } else {
                syn::parse_quote! {
                    where
                    MemoryView: ::raw_struct::MemoryView + 'static
                }
            };

            generics
        };

        let instance_name = Ident::new(&format!("{}_Instance", &target.ident), target.ident.span());

        Ok(Self {
            source: target,
            args: target_args,

            attributes,
            fields,

            generics_impl,
            generics_type,
            generics_where,

            accessor_name,
            accessor_generics,

            instance_name,
        })
    }

    /// Generate the main trait named like the original struct
    /// implementing the Viewable trait.
    fn generate_main_trait(&self) -> TokenStream {
        let vis = &self.source.vis;
        let name = &self.source.ident;
        let name_str = self.source.ident.to_string();
        let attrs = &self.attributes;

        let generics_impl = &self.generics_impl;
        let generics_type = &self.generics_type;
        let generics_where = &self.generics_where;

        let memory = &self.args.memory;

        let accessor_name = &self.accessor_name;
        let (_, accessor_type_generics, _) = self.accessor_generics.split_for_impl();

        let instance_name = &self.instance_name;

        quote! {
            #(#attrs)*
            #vis trait #name #generics_type #generics_where { }

            impl #generics_impl ::raw_struct::Viewable for dyn #name #generics_type #generics_where {
                type Memory = #memory;

                type Accessor<MemoryView: ::raw_struct::MemoryView + 'static> = dyn #accessor_name #accessor_type_generics;

                type Instance<MemoryView: ::raw_struct::MemoryView + 'static> = #instance_name <Self::Accessor<MemoryView>, MemoryView>;

                fn name() -> ::raw_struct::Cow<'static, str> {
                    concat!(module_path!(), "::", #name_str).into()
                }

                fn create_view<M: ::raw_struct::MemoryView + 'static>(memory: M) -> Self::Instance<M> {
                    #instance_name { _accessor: Default::default(), memory }
                }
            }
        }
    }

    fn generate_accessor_trait(&self) -> TokenStream {
        let vis = &self.source.vis;
        let name = &self.accessor_name;
        let accessors = self
            .fields
            .iter()
            .filter_map(|field| field.generate_accessor(self));

        let (_accessor_impl_generics, accessor_ty_generics, accessor_where_clause) =
            self.accessor_generics.split_for_impl();

        quote! {
            #[allow(non_camel_case_types)]
            #vis trait #name #accessor_ty_generics : ::raw_struct::ViewableBase<MemoryView>
                #accessor_where_clause
            {
                #(#accessors)*
            }
        }
    }

    fn generate_instance_struct(&self) -> TokenStream {
        let instance_name = &self.instance_name;
        let accessor_name = &self.accessor_name;

        let (accessor_impl_generics, accessor_ty_generics, accessor_where_clause) =
            self.accessor_generics.split_for_impl();

        quote! {
            #[allow(non_camel_case_types)]
            struct #instance_name <A: ?Sized, M: ::raw_struct::MemoryView> {
                _accessor: ::core::marker::PhantomData<A>,
                memory: M,
            }

            impl<A: ?Sized + Send + Sync, M: ::raw_struct::MemoryView> ::raw_struct::ViewableBase<M> for #instance_name<A, M> {
                fn object_memory(&self) -> &M {
                    &self.memory
                }
            }

            impl #accessor_impl_generics ::raw_struct::ViewableInstance<dyn #accessor_name #accessor_ty_generics, MemoryView>
                for #instance_name<dyn #accessor_name #accessor_ty_generics, MemoryView>
                #accessor_where_clause
            {
                fn get_accessor(&self) -> &(dyn #accessor_name #accessor_ty_generics + 'static) {
                    self
                }
            }

            impl #accessor_impl_generics #accessor_name #accessor_ty_generics
                for #instance_name<dyn #accessor_name #accessor_ty_generics, MemoryView>
                #accessor_where_clause { }


            impl<A: ?Sized, MemoryView: ::raw_struct::MemoryView + Clone> Clone
            for #instance_name<A, MemoryView>  {
                fn clone(&self) -> Self {
                    Self {
                        _accessor: Default::default(),
                        memory: self.memory.clone(),
                    }
                }
            }

            impl <A: ?Sized, MemoryView: ::raw_struct::MemoryView + ::core::marker::Copy> ::core::marker::Copy
                for #instance_name<A, MemoryView> { }
        }
    }
}

pub fn raw_struct(attr: TokenStream, input: TokenStream) -> Result<TokenStream> {
    let args = syn::parse2::<StructArgs>(attr)?;
    let target = syn::parse2::<ItemStruct>(input)?;

    let generator = ViewableGenerator::new(&target, args)?;
    // println!(
    //     "/* Main trait: */ {}",
    //     generator.generate_main_trait().to_string()
    // );

    // println!(
    //     "/* Accessor trait: */ {}",
    //     generator.generate_accessor_trait().to_string()
    // );

    // println!(
    //     "/* Instance struct: */ {}",
    //     generator.generate_instance_struct().to_string()
    // );

    let main_trait = generator.generate_main_trait();
    let accessors = generator.generate_accessor_trait();
    let instance = generator.generate_instance_struct();
    Ok(quote! {
        #main_trait
        #accessors
        #instance
    })
}
