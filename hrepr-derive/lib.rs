extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::format_ident;
use quote::quote;
use syn::Data;
use syn::DeriveInput;
use syn::Fields;
use syn::parse_macro_input;

fn mk_idx(i: usize) -> TokenStream2 {
    (0..i).fold(quote! {::hrepr::Zero}, |tail, _| quote! {::hrepr::Succ<#tail>})
}

fn mk_type_var(i: usize) -> TokenStream2 {
    let idx = mk_idx(i);
    quote! {::hrepr::TypeVar<#idx>}
}

#[proc_macro_derive(Generic)]
pub fn derive_generic(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let type_ident = &input.ident;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let type_params =
        input.generics.params.iter().rfold(quote! {()}, |tail, field| quote! {(#field, #tail)});
    let mock_params = input.generics.type_params().enumerate().map(|(i, _)| mk_type_var(i));

    let (repr, to_repr, from_repr) = match &input.data {
        Data::Struct(str) => match &str.fields {
            Fields::Named(fields) => {
                let idents = fields.named.iter().map(|field| field.ident.as_ref().unwrap());

                let types_hlist = fields.named.iter().map(|field| &field.ty).rfold(
                    quote! {::hrepr::HNil},
                    |tail, field| quote! {::hrepr::HCons<#field, #tail>},
                );
                let to_repr = idents.clone().rfold(
                    quote! {::hrepr::HNil},
                    |tail, field_name| quote! {::hrepr::HCons(self.#field_name, #tail)},
                );
                let from_repr_pat = idents.clone().rfold(
                    quote! {::hrepr::HNil},
                    |tail, field_name| quote! {::hrepr::HCons(#field_name, #tail)},
                );
                let from_repr = quote! {let ::hrepr::NamedStruct(#from_repr_pat) = repr; Self { #(#idents,)* } };
                (
                    quote! {::hrepr::NamedStruct<#types_hlist>},
                    quote! {::hrepr::NamedStruct(#to_repr)},
                    from_repr,
                )
            }
            Fields::Unnamed(fields) => {
                let indexes = fields.unnamed.iter().enumerate();

                let types_hlist = fields.unnamed.iter().map(|field| &field.ty).rfold(
                    quote! {::hrepr::HNil},
                    |tail, field| quote! {::hrepr::HCons<#field, #tail>},
                );
                let to_repr = indexes.clone().map(|(i, _)| syn::Index::from(i)).rfold(
                    quote! {::hrepr::HNil},
                    |tail, i| quote! {::hrepr::HCons(self.#i, #tail)},
                );

                let idents = indexes.map(|(i, _)| format_ident!("x{i}"));

                let from_repr_pat = idents
                    .clone()
                    .rfold(quote! {::hrepr::HNil}, |tail, i| quote! {::hrepr::HCons(#i, #tail)});

                let from_repr = quote! {let ::hrepr::UnnamedStruct(#from_repr_pat) = repr; Self ( #(#idents,)* ) };
                (
                    quote! {::hrepr::UnnamedStruct<#types_hlist>},
                    quote! {::hrepr::UnnamedStruct(#to_repr)},
                    from_repr,
                )
            }
            Fields::Unit => {
                (quote! {::hrepr::UnitStruct}, quote! {::hrepr::UnitStruct}, quote! {Self})
            }
        },
        Data::Enum(data_enum) => {
            let repr = data_enum.variants.iter().map(|variant| &variant.fields).rfold(
                quote! {::hrepr::HSumNil},
                |tail, var_fields| {
                    let var_struct = var_fields.iter().map(|field| &field.ty).rfold(
                        quote! {::hrepr::HNil},
                        |tail, field| quote! {::hrepr::HCons<#field, #tail>},
                    );

                    match &var_fields {
                        Fields::Named(_) => {
                            quote! {::hrepr::HSum<::hrepr::NamedStruct<#var_struct>, #tail>}
                        }
                        Fields::Unnamed(_) => {
                            quote! {::hrepr::HSum<::hrepr::UnnamedStruct<#var_struct>, #tail>}
                        }
                        Fields::Unit => quote! {::hrepr::HSum<::hrepr::UnitStruct, #tail>},
                    }
                },
            );

            let to_from_repr = data_enum.variants.iter().enumerate().map(|(i, variant)| {
                let var_ident = &variant.ident;
                let (orig, generic) = match &variant.fields {
                    Fields::Named(fields) => {
                        let field_idents =
                            fields.named.iter().map(|field| field.ident.as_ref().unwrap());
                        let cons = field_idents.clone().rfold(
                            quote! {::hrepr::HNil},
                            |tail, field_ident| quote! {::hrepr::HCons(#field_ident, #tail)},
                        );
                        (quote! {{#(#field_idents,)*}}, quote! {::hrepr::NamedStruct(#cons)})
                    }
                    Fields::Unnamed(fields) => {
                        let indices =
                            fields.unnamed.iter().enumerate().map(|(j, _)| format_ident!("x{j}"));
                        let cons = indices.clone().rfold(
                            quote! {::hrepr::HNil},
                            |tail, j| quote! {::hrepr::HCons(#j, #tail)},
                        );
                        (quote! {(#(#indices,)*)}, quote! {::hrepr::UnnamedStruct(#cons)})
                    }
                    Fields::Unit => (quote! {}, quote! {::hrepr::UnitStruct}),
                };
                let generic = (0..i).fold(
                    quote! {::hrepr::HSum::L(#generic)},
                    |tail, _| quote! {::hrepr::HSum::R(#tail)},
                );
                (var_ident, orig, generic)
            });

            let to_repr_matches = to_from_repr
                .clone()
                .map(|(var_ident, orig, generic)| quote! {Self::#var_ident #orig => #generic});
            let to_repr = quote! { ::hrepr::Enum(match self { #(#to_repr_matches,)* }) };

            let from_repr_matches = to_from_repr
                .map(|(var_ident, orig, generic)| quote! {#generic => Self::#var_ident #orig });
            let from_repr = quote! { match repr.0 { #(#from_repr_matches,)* } };

            (quote! {::hrepr::Enum<#repr>}, to_repr, from_repr)
        }
        Data::Union(_) => panic!("Union types are not supported"),
    };

    quote! {
      #[automatically_derived]
      impl #impl_generics ::hrepr::Generic for #type_ident #ty_generics #where_clause {
        type Params = #type_params;
        type Mocked = #type_ident <#(#mock_params,)*>;
        type Repr = #repr;
        fn to_repr(self) -> Self::Repr { #to_repr }
        fn from_repr(repr: Self::Repr) -> Self { #from_repr }
      }
    }
    .into()
}
