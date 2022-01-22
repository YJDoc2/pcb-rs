mod chip_derive;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Chip, attributes(pin))]
pub fn derive_chip(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    match &ast.data {
        syn::Data::Enum(_) | syn::Data::Union(_) => {
            panic!("Chip derive is only supported for structs")
        }

        syn::Data::Struct(chip_struct) => {
            return chip_derive::derive_chip_impl(&ast.ident, chip_struct).into();
        }
    }
}
