mod chip_derive;
mod pcb_macro;

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

#[proc_macro]
pub fn pcb(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as pcb_macro::PcbMacroInput);
    let output: proc_macro2::TokenStream = input.into();
    output.into()
}
