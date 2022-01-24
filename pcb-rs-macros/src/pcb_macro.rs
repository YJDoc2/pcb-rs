use std::collections::{HashMap, HashSet};
use syn::parse::{Parse, ParseStream};
use syn::{Result, Token};

// ! TODO Add better error reporting

const CHIP_DEFINITION_KEYWORD: &str = "chip";
const PIN_EXPOSE_KEYWORD: &str = "expose";

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct ChipPin {
    chip: String,
    pin: String,
}

#[derive(Debug)]
pub struct PcbMacroInput {
    name: syn::Ident,
    chip_map: HashMap<String, Vec<String>>,
    pin_connection_list: HashMap<ChipPin, HashSet<ChipPin>>,
    exposed_pins: Vec<ChipPin>,
}

impl Parse for PcbMacroInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = syn::Ident::parse(input)?;
        let content;
        let _braces = syn::braced!(content in input);
        let mut kw;
        let mut chip_map: HashMap<String, Vec<String>> = HashMap::new();

        // this just stores a simple representation of connected pins,
        // we convert this into a better structure to store into the builder in the into function
        let mut pin_connection_list: HashMap<ChipPin, HashSet<ChipPin>> = HashMap::new();

        let mut exposed_pins: Vec<ChipPin> = Vec::new();

        // this parses the module
        loop {
            kw = syn::Ident::parse(&content)?;
            if kw != CHIP_DEFINITION_KEYWORD {
                break;
            }
            let module_name = syn::Ident::parse(&content)?;
            let _ = <Token![;]>::parse(&content)?;
            chip_map.insert(module_name.to_string(), Vec::new());
        }

        if chip_map.is_empty() {
            panic!("cannot make pcb with no chips!");
        }

        if kw == PIN_EXPOSE_KEYWORD {
            panic!("there are no pin connections in this pcb!");
        }

        // here the kw will actually point to name of chip, for pin connections
        loop {
            if kw == PIN_EXPOSE_KEYWORD {
                break;
            }
            let chip1 = kw.to_string();
            let _ = <Token![::]>::parse(&content)?;
            let pin1 = syn::Ident::parse(&content)?.to_string();
            // pin connection token is -
            let _ = <Token![-]>::parse(&content);
            let chip2 = syn::Ident::parse(&content)?.to_string();
            let _ = <Token![::]>::parse(&content)?;
            let pin2 = syn::Ident::parse(&content)?.to_string();
            let _ = <Token![;]>::parse(&content)?;

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

            let chip_pin1 = ChipPin {
                chip: chip1,
                pin: pin1,
            };
            let chip_pin2 = ChipPin {
                chip: chip2,
                pin: pin2,
            };

            if let Some(l) = pin_connection_list.get_mut(&chip_pin1) {
                // we first check if pin1 is already an entry, if so then add pin2 to its set
                l.insert(chip_pin2);
            } else if let Some(l) = pin_connection_list.get_mut(&chip_pin2) {
                // else we check if pin2 is already an entry
                l.insert(chip_pin1);
            } else {
                let mut _t = HashSet::new();
                _t.insert(chip_pin2);
                pin_connection_list.insert(chip_pin1, _t);
            }

            // we have to parse it here for the next iteration

            match syn::Ident::parse(&content) {
                Result::Ok(i) => kw = i,
                Result::Err(_) => {
                    return Ok(PcbMacroInput {
                        name,
                        pin_connection_list,
                        chip_map,
                        exposed_pins: Vec::new(),
                    })
                }
            }
        }
        // now here the kw should be exposed
        loop {
            let chip = syn::Ident::parse(&content)?.to_string();
            let _ = <Token![::]>::parse(&content)?;
            let pin = syn::Ident::parse(&content)?.to_string();
            let _ = <Token![;]>::parse(&content);
            if !chip_map.contains_key(&chip) {
                panic!("use of undeclared chip in expose pin : {}", chip);
            }
            exposed_pins.push(ChipPin {
                chip: chip,
                pin: pin,
            });
            match syn::Ident::parse(&content) {
                Result::Ok(i) => {
                    if i != PIN_EXPOSE_KEYWORD {
                        panic!("expected 'expose' found {} instead", i.to_string());
                    }
                }
                // this just means we have completed the parsing
                Result::Err(_) => break,
            }
        }

        Ok(PcbMacroInput {
            name,
            pin_connection_list,
            chip_map,
            exposed_pins,
        })
    }
}

impl Into<proc_macro2::TokenStream> for PcbMacroInput {
    fn into(self) -> proc_macro2::TokenStream {
        proc_macro2::TokenStream::new()
        // "fn test()->usize{5}".parse().unwrap()
    }
}
