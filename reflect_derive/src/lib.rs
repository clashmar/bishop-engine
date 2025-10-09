// reflect_derive/src/lib.rs
extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, 
    Data, 
    DeriveInput, 
    Fields,
};

/// `#[derive(Reflect)]` – generates an impl of the `Reflect` trait.
#[proc_macro_derive(Reflect)]
pub fn derive_reflect(input: TokenStream) -> TokenStream {
    // Parse the input token stream into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident; // Struct name
    let generics = input.generics; // keep generic params untouched

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
            return syn::Error::new_spanned(
                name,
                "Reflect can only be derived for structs",
            )
            .to_compile_error()
            .into();
        }
    };

    // For each field generate a call to <FieldType>::field_info(...)
    let field_infos = fields.iter().map(|f| {
        let field_name = f.ident.as_ref().unwrap(); // e.g. `damage`
        let field_str = field_name.to_string(); // e.g. "damage"
        let ty = &f.ty; // the field type

        // The macro simply emits:
        //   <Ty as ReflectField>::field_info(&mut self.<field>, "field")
        quote! {
            <#ty as crate::ecs::reflect::ReflectField>::field_info(
                &mut self.#field_name,
                #field_str
            )
        }
    });

    // Build the final impl block
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let expanded = quote! {
        // The generated impl lives in the same crate that defines `Reflect`
        impl #impl_generics crate::ecs::reflect::Reflect for #name #ty_generics #where_clause {
            fn fields(&mut self) -> ::std::vec::Vec<crate::ecs::reflect::FieldInfo<'_>> {
                vec![
                    #(#field_infos),*
                ]
            }
        }
    };

    // Hand the generated code back to the compiler
    TokenStream::from(expanded)
}