use quote::quote;
use std::collections::{HashMap, HashSet};
use syn::parse::{Parse, ParseStream};
use syn::{Result, Token};
// ! TODO Add better error reporting

const CHIP_DEFINITION_KEYWORD: &str = "chip";
const PIN_EXPOSE_KEYWORD: &str = "expose";

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct __ChipPin {
    chip: String,
    pin: String,
}

#[derive(Debug)]
pub struct PcbMacroInput {
    name: syn::Ident,
    chip_map: HashMap<String, Vec<String>>,
    pin_connection_list: HashMap<__ChipPin, HashSet<__ChipPin>>,
    exposed_pins: Vec<__ChipPin>,
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
        let mut pin_connection_list: HashMap<__ChipPin, HashSet<__ChipPin>> = HashMap::new();

        let mut exposed_pins: Vec<__ChipPin> = Vec::new();

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
           return Err(syn::Error::new_spanned(&name,"cannot make pcb with no chips!"));
            
        }

        if kw == PIN_EXPOSE_KEYWORD {
            return Err(syn::Error::new_spanned(&name,"there are no pin connections in this pcb!"));   
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

            if (&chip1,&pin1) == (&chip2,&pin2){
                let t =format!("attempted to connect a pin to itself : chip `{}` pin `{}` appears to have a self-connection, which is redundant",chip1,pin1);
                return Err(syn::Error::new_spanned(&chip1,t));
            }

            if !chip_map.contains_key(&chip1) {
                let t = format!("use of undeclared chip {}", chip1);
                return Err(syn::Error::new_spanned(&chip1,t));
            }

            if !chip_map.contains_key(&chip2) {
                let t = format!("use of undeclared chip {}", chip2);
                return Err(syn::Error::new_spanned(&chip2,t));
            }

            // now we know for sure that both chips are declared and exists in the map

            let t = chip_map.get_mut(&chip1).unwrap();
            t.push(pin1.clone());
            let t = chip_map.get_mut(&chip2).unwrap();
            t.push(pin2.clone());

            let chip_pin1 = __ChipPin {
                chip: chip1,
                pin: pin1,
            };
            let chip_pin2 = __ChipPin {
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
                let t = format!("use of undeclared chip in expose pin : {}", chip);
                return Err(syn::Error::new_spanned(&chip,t));
            }
            exposed_pins.push(__ChipPin {
                chip: chip,
                pin: pin,
            });
            match syn::Ident::parse(&content) {
                Result::Ok(i) => {
                    if i != PIN_EXPOSE_KEYWORD {
                        let t =format!("expected 'expose' found {} instead", i.to_string());
                        return Err(syn::Error::new_spanned(i,t));
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
        self.generate()
    }
}

impl PcbMacroInput {


    // This might be more efficiently implemented, I think this has worst case O(n^2)?
    fn get_short_pin_set(&self)->Vec<Vec<__ChipPin>>{
        // first let us make a vec to store the initial pin connections
        // generated from given param
        let mut initial_collection = Vec::new();

        // we fill that vec by pushing sets in the param, but adding the hashmap key to the set
        // as we need set of connected pin, and the key-value structure doesn't make a difference
        for (main,connected) in &self.pin_connection_list{
            let mut t = connected.clone();
            t.insert(main.clone());
            initial_collection.push(t);
        }

        // this is the final return, which is the collection of groups of all the pins that are 
        // shorted, i.e. connected electrically, so that voltage at any
        // one fo the pins in the individual group will affect rest of the pins in that group
        let mut shorted_pins = Vec::new();

        loop{

            // we take a set from the initial sets, if no sets are remaining,
            // work is done
            let mut set = match initial_collection.pop(){
                Some(s)=>s,
                None=>break
            };
            // a temp vector to store the groups which does not have any pins in common
            // with the set above
            let mut t = Vec::new();

            // we check if any remaining set in the initial collection
            // has a pin common with the set under consideration,
            // if it does, we extend the set, else store that (remaining) set in 
            // the temp vector
            for s in initial_collection{
                if set.intersection(&s).next().is_some(){
                    set.extend(s.into_iter());
                }else{
                    t.push(s);
                }
            }

            // not the set contains pins which are shorted, we store that in the return variable
            shorted_pins.push(set.into_iter().collect());

            // set the initial collection to temp, so it contains next candidates to check
            initial_collection = t;
        }

        // return shorted pins
        shorted_pins
    }

    fn generate(self) -> proc_macro2::TokenStream {
        let pcb_name = &self.name;
        let builder_name = quote::format_ident!("{}Builder", pcb_name);

        let chip_names = self.chip_map.iter().map(|(name, _)| quote! {#name});

        let chip_pin_check = self.chip_map.iter().map(|(name,pins)|{
            let pin_names = pins.iter().map(|n|{quote!{#n}});
            quote!{
                let chip = self.added_chip_map.get(#name).unwrap();
                let chip_pins = chip.get_pin_list();
                for pin in [#(#pin_names),*]{
                    if !chip_pins.contains_key(pin){
                        return std::result::Result::Err(format!("Invalid chip added : chip {} expected to have pin named {}, not found",#name,pin));
                    }
                }
            }
        });

        // this will bind some variables to the actual entered chips
        let instantiate_chip_vars = self.chip_map.iter().map(|(name, _)| {
            let __name = syn::Ident::new(&name, pcb_name.span());
            quote! {let #__name = self.added_chip_map.get(#name).unwrap().get_pin_list();}
        });
        

        let pin_connection_checks = self
            .pin_connection_list
            .iter()
            .map(|(pin, connected_pins)| {
                let _chip = &pin.chip;
                let _pin = &pin.pin;
                let chip_ident = syn::Ident::new(&_chip, pcb_name.span());
                let connected_pin_iter = connected_pins.iter().map(|pin| {
                    let __chip = &pin.chip;
                    let __pin = &pin.pin;
                    let chip_ident = syn::Ident::new(__chip,pcb_name.span());
                    quote! {
                        let __pin2 = #chip_ident.get(#__pin).unwrap();
                        if !__pin1.is_connectable(__pin2){
                            return std::result::Result::Err(
                                format!("Invalid chip connection : cannot connect chip {}'s pin {} ({:?}) to chip {}'s pin {} ({:?})",
                                    #_chip,#_pin,__pin1,#__chip,#__pin,__pin2
                                )
                            )
                        }
                    }
                });

                quote! {
                    let __pin1 = #chip_ident.get(#_pin).unwrap();
                    #(#connected_pin_iter)*   
                }
            });

        let shorted_pins = self.get_short_pin_set();
        let shorted_pins_tokens = shorted_pins.iter().map(|group|{
            let g = group.iter().map(|cp|{
                let chip = &cp.chip;
                let pin= &cp.pin;
                quote!{
                    pcb_rs::ChipPin{
                        chip:#chip,
                        pin:#pin
                    }
                }
            });
            quote!{
                std::vec![#(#g),*]
            }
        });
        

        quote! {

            struct #pcb_name{}

            struct #builder_name{
                added_chip_map:std::collections::HashMap<std::string::String,std::boxed::Box<dyn pcb_rs::HardwareModule>>,
                shorted_pins:std::vec::Vec<std::vec::Vec<pcb_rs::ChipPin>>
            }

            impl #builder_name{

                pub fn new()->Self{
                    let shorted = std::vec![#(#shorted_pins_tokens),*];
                    Self{
                        added_chip_map:std::collections::HashMap::new(),
                        shorted_pins:shorted,
                    }
                }

                pub fn add_chip(mut self,name:&str,chip: std::boxed::Box<dyn pcb_rs::HardwareModule>)->Self{
                    self.added_chip_map.insert(name.to_string(),chip);
                    self
                }

                pub fn build(self)->std::result::Result<#pcb_name, std::string::String>{
                    self.check_added_all_chips()?;
                    self.check_valid_chips()?;
                    self.check_valid_pin_connection()?;

                    std::result::Result::Ok(#pcb_name{})
                }

                fn check_added_all_chips(&self)-> std::result::Result<(),std::string::String>{
                    for chip in [#(#chip_names),*]{
                        if !self.added_chip_map.contains_key(chip){
                            return std::result::Result::Err(format!("chip {} defined in pcb design, but not added",chip))
                        }
                    }
                    std::result::Result::Ok(())
                }
                fn check_valid_chips(&self)-> std::result::Result<(),std::string::String>{
                    #(#chip_pin_check)*
                    std::result::Result::Ok(())
                }

                fn check_valid_pin_connection(&self)->std::result::Result<(),std::string::String>{
                    #(#instantiate_chip_vars)*
                    #(#pin_connection_checks)*
                    
                    std::result::Result::Ok(())
                }

            }
        }
    }
}
