// Copyright 2021 Axiom-Team
//
// This file is part of Duniter-v2S.
//
// Duniter-v2S is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, version 3 of the License.
//
// Duniter-v2S is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with Duniter-v2S. If not, see <https://www.gnu.org/licenses/>.

#![crate_type = "proc-macro"]
extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, spanned::Spanned, Fields, Ident, ItemStruct, Type};

fn snake_to_class_case(ident: &Ident) -> Ident {
    let span = ident.span();
    let mut acc = String::new();
    let mut prev = '_';
    for ch in ident.to_string().chars() {
        if ch != '_' {
            if prev == '_' {
                for chu in ch.to_uppercase() {
                    acc.push(chu);
                }
            } else if prev.is_uppercase() {
                for chl in ch.to_lowercase() {
                    acc.push(chl);
                }
            } else {
                acc.push(ch);
            }
        }
        prev = ch;
    }
    Ident::new(&acc, span)
}

#[proc_macro_attribute]
pub fn generate_fields_getters(_: TokenStream, input: TokenStream) -> TokenStream {
    let item_struct = parse_macro_input!(input as ItemStruct);

    let ItemStruct { fields, .. } = item_struct.clone();

    let mut class_idents: Vec<Ident> = Vec::new();
    let mut idents: Vec<Ident> = Vec::new();
    let mut types: Vec<Type> = Vec::new();
    if let Fields::Named(named_fields) = fields {
        let named_fields = named_fields.named;
        for field in named_fields.iter() {
            if let Some(ref ident) = field.ident {
                class_idents.push(snake_to_class_case(ident));
                idents.push(ident.clone());
                types.push(field.ty.clone());
            }
        }

        (quote! {
            #item_struct

            #(
                pub struct #class_idents<T: Config>(core::marker::PhantomData<T>);
                impl<T: Config> Get<T::#types> for #class_idents<T> {
                    fn get() -> T::#types {
                        Pallet::<T>::parameters().#idents
                    }
                }
            )*
        })
        .into()
    } else {
        (quote_spanned! {
            fields.span() => compile_error("Expected named fields");
        })
        .into()
    }
}
