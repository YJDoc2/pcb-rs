use std::collections::HashMap;
use syn::parse::{Parse, ParseStream};
use syn::{Result, Token};

// ! TODO Add better error reporting

const CHIP_DEFINITION_KEYWORD: &str = "chip";
const PIN_EXPOSE_KEYWORD: &str = "expose";

#[derive(Debug)]
pub struct PcbMacroInput {}

impl Parse for PcbMacroInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = syn::Ident::parse(input)?;
        let content;
        let _braces = syn::braced!(content in input);
        let mut module_kw;
        let mut chip_map: HashMap<String, Vec<String>> = HashMap::new();

        // this parses the module
        loop {
            module_kw = syn::Ident::parse(&content)?;
            if module_kw != CHIP_DEFINITION_KEYWORD {
                break;
            }
            let module_name = syn::Ident::parse(&content)?;
            let _ = <Token![;]>::parse(&content)?;
            chip_map.insert(module_name.to_string(), Vec::new());
        }

        dbg!(&module_map);
        if chip_map.is_empty() {
            panic!("cannot make pcb with no chips!");
        }

        if module_kw == PIN_EXPOSE_KEYWORD {
            panic!("there are no pin connections in this pcb!");
        }

        // here the module_kw will actually point to name of chip, for pin connections
        loop {
            if module_kw == PIN_EXPOSE_KEYWORD {
                break;
            }
            let chip1 = module_kw.to_string();
            let _ = <Token![::]>::parse(&content)?;
            let pin1 = syn::Ident::parse(&content)?.to_string();
            // pin connection token is -
            let _ = <Token![-]>::parse(&content);
            let chip2 = syn::Ident::parse(&content)?.to_string();
            let pin2 = syn::Ident::parse(&content)?.to_string();

            if !chip_map.contains_key(&chip1) {
                panic!("use of undeclared chip {}", chip1);
            }

            if !chip_map.contains_key(&chip2) {
                panic!("use of undeclared chip {}", chip2);
            }

            // now we know for sure that both chips are declared and exists in the map

            let t = chip_map.get_mut(&chip1).unwrap();
            t.push(pin1.clone());
            let t = chip_map.get_mut(&chip2).unwrap();
            t.push(pin2.clone());

            // now here we have to think of how to represent and store the connection of the chips

            // we have to parse it here for the next iteration
            module_kw = syn::Ident::parse(&content)?;
        }

        Ok(PcbMacroInput {})
    }
}

impl Into<proc_macro2::TokenStream> for PcbMacroInput {
    fn into(self) -> proc_macro2::TokenStream {
        proc_macro2::TokenStream::new()
        // "fn test()->usize{5}".parse().unwrap()
    }
}
