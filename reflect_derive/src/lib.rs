// reflect_derive/src/lib.rs
extern crate proc_macro;
use syn::parse_macro_input;
use proc_macro::TokenStream;
use syn::DeriveInput;
use syn::Attribute;
use quote::quote;
use syn::LitStr;
use syn::Fields;
use syn::Token;
use syn::Data;

/// `#[derive(Reflect)]` – generates an impl of the `Reflect` trait.
#[proc_macro_derive(Reflect, attributes(widget))]
pub fn derive_reflect(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let generics = input.generics;

    // Collect field information (only works for structs with named fields)
    let fields = match input.data {
        Data::Struct(s) => match s.fields {
            Fields::Named(named) => named.named,
            _ => {
                return syn::Error::new_spanned(
                    s.struct_token,
                    "Reflect can only be derived for structs with named fields",
                )
                .to_compile_error()
                .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(name, "Reflect can only be derived for structs")
                .to_compile_error()
                .into();
        }
    };

    let field_infos = fields.iter().map(|f| {
        let field_name = f.ident.as_ref().unwrap();
        let field_str = field_name.to_string();
        let ty = &f.ty;

        let hint_opt = widget_hint(&f.attrs);
        let hint_expr = match hint_opt {
            Some(s) => quote! { Some(#s) },
            None => quote! { None },
        };

        quote! {
            <#ty as crate::ecs::reflect_field::ReflectField>::field_info(
                &mut self.#field_name,
                #field_str
            )
            .with_hint(#hint_expr)
        }
    });

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics crate::ecs::reflect_field::Reflect for #name #ty_generics #where_clause {
            fn fields(&mut self) -> ::std::vec::Vec<crate::ecs::reflect_field::FieldInfo<'_>> {
                vec![
                    #(#field_infos),*
                ]
            }
        }
    };

    TokenStream::from(expanded)
}

/// Returns the string literal that appears in `#[widget = "…"]`.
/// If the attribute is missing, has a different name, or the value is not a
/// string literal, `None` is returned.
fn widget_hint(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        if !attr.path().is_ident("widget") {
            continue;
        }

        // Parse the token stream after the path
        let parser = |input: syn::parse::ParseStream| {
            if input.peek(Token![=]) {
                let _eq: Token![=] = input.parse()?;
            }
            let lit: LitStr = input.parse()?;
            Ok(lit)
        };

        // If parsing fails we just ignore the attribute
        if let Ok(lit) = attr.parse_args_with(parser) {
            return Some(lit.value());
        }
    }
    None
}