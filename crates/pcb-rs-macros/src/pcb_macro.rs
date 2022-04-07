use quote::quote;
use std::collections::{HashMap, HashSet};
use syn::parse::{Parse, ParseStream};
use syn::{Result, Token};
// ! TODO Add better error reporting
// ! TODO maybe refactor the pin validation fn, where it also sets the pin metadata?

const CHIP_DEFINITION_KEYWORD: &str = "chip";
const PIN_EXPOSE_KEYWORD: &str = "expose";

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct __ChipPin {
    chip: String,
    pin: String,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct __ExposedPins{
    pins:Vec<__ChipPin>,
    as_name:String
}

#[derive(Debug)]
pub struct PcbMacroInput {
    name: syn::Ident,
    chip_map: HashMap<String, Vec<String>>,
    pin_connection_list: HashMap<__ChipPin, HashSet<__ChipPin>>,
    exposed_pins: Vec<__ExposedPins>,
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

        let mut exposed_pins: Vec<__ExposedPins> = Vec::new();

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

        // we allow pcb with no connections as one might use pcb as a
        // convenient collection of chips

        // here the kw will actually point to name of chip, for pin connections
        // unless there are no connections, in which case it will break on first iter
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
                let t = format!("attempted to connect a pin to itself : chip `{}` pin `{}` appears to have a self-connection, which is redundant",chip1,pin1);
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
            if !chip_map.contains_key(&chip) {
                let t = format!("use of undeclared chip in expose pin : {}", chip);
                return Err(syn::Error::new_spanned(&chip,t));
            }

            let mut pins = vec![__ChipPin{
                chip,pin
            }];
            // if we have something like `expose c1::p1,c2::p1 as p3;`
            if content.peek(Token![,]){
                // we do have it like `expose c1::p1,c2::p1 as p3;`
                loop{
                    let _ = <Token![,]>::parse(&content)?;
                    let chip = syn::Ident::parse(&content)?.to_string();
                        let _ = <Token![::]>::parse(&content)?;
                        let pin = syn::Ident::parse(&content)?.to_string();
                        if !chip_map.contains_key(&chip) {
                            let t = format!("use of undeclared chip in expose pin : {}", chip);
                            return Err(syn::Error::new_spanned(&chip,t));
                        }
                        pins.push(__ChipPin{chip,pin});
                        // if we have a comma, there are more pins, else we can exit the loop
                        if content.peek(Token![,]){
                            continue;
                        }else{
                            break;
                        }
                }
            }
            // now there must be an `as` keyword
            let _ = <Token![as]>::parse(&content);
            let as_name = syn::Ident::parse(&content)?.to_string();
            let _ = <Token![;]>::parse(&content);
            
            exposed_pins.push(__ExposedPins { pins, as_name });
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

        let mut temp = HashMap::new();
        for ep in &exposed_pins{
            for p in &ep.pins{
                if temp.contains_key(p){
                    let previous = temp.get(p).unwrap();
                    let t = format!("pin exposed multiple times : chip {} pin {} is exposed as {} and {}",
                        p.chip,p.pin,previous, ep.as_name
                    );
                    return Err(syn::Error::new_spanned(&name,t));
                }else{
                    temp.insert(p,&ep.as_name);
                }
                    
                
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
            if pins.is_empty(){
                return quote!{};
            }
            let pin_names = pins.iter().map(|n|{quote!{#n}});
            let exposed_pins:Vec<_> = self.exposed_pins.iter()
                .flat_map(|ep|&ep.pins)
                .filter(|p|{p.chip == *name})
                .map(|cp|&cp.pin)
                .map(|n|quote!{#n})
                .collect();
            let exposed_pin_check = if exposed_pins.len() == 0{
                quote!{}
            }else{
                let names = exposed_pins.iter();
                quote!{
                    for pin in [#(#names),*]{
                        if !chip_pins.contains_key(pin){
                            return std::result::Result::Err(format!("Invalid chip added : chip {} expected to have pin named {}, not found",#name,pin));
                        }
                    }
                }
            };
            quote!{
                let chip = self.added_chip_map.get(#name).unwrap();
                let chip_pins = chip.get_pin_list();
                for pin in [#(#pin_names),*]{
                    if !chip_pins.contains_key(pin){
                        return std::result::Result::Err(format!("Invalid chip added : chip {} expected to have pin named {}, not found",#name,pin));
                    }
                }
                #exposed_pin_check
            }
        });

        // TODO improve this!
        // this will bind some variables to the actual entered chips for the builder
        let instantiate_chip_vars = self.chip_map.iter().map(|(name, _)| {
            if self.pin_connection_list.is_empty(){
                quote!{}
            }else{
                
                let __name = quote::format_ident!("_{}",&name);
                quote! {let #__name = self.added_chip_map.get(#name).unwrap().get_pin_list();}
            }
        });

        

        let pin_connection_checks = self
            .pin_connection_list
            .iter()
            .map(|(pin, connected_pins)| {
                let _chip = &pin.chip;
                let _pin = &pin.pin;
                let chip_ident = quote::format_ident!("_{}",_chip);
                let connected_pin_iter = connected_pins.iter().map(|pin| {
                    let __chip = &pin.chip;
                    let __pin = &pin.pin;
                    let chip_ident = quote::format_ident!("_{}",__chip);
                    quote! {
                        let __pin2 = #chip_ident.get(#__pin).unwrap();
                        if !__pin1.is_connectable(__pin2){
                            return std::result::Result::Err(
                                format!("Invalid chip connection : cannot connect chip {}'s pin {} ({:?}) to chip {}'s pin {} ({:?})",
                                    #_chip,#_pin,__pin1,#__chip,#__pin,__pin2
                                )
                            )
                        }
                        self.pin_metadata_cache.insert(pcb_rs::ChipPin{
                            chip: #__chip,
                            pin: #__pin,
                        },*__pin2);
                    }
                });

                quote! {
                    let __pin1 = #chip_ident.get(#_pin).unwrap();
                    self.pin_metadata_cache.insert(pcb_rs::ChipPin{
                        chip:#_chip,
                        pin:#_pin
                    },*__pin1);
                    #(#connected_pin_iter)*   
                }
            });

        let shorted_pins = self.get_short_pin_set();

        // TODO maybe move this to the parsing stage?
        for ep in &self.exposed_pins{
            let pins = &ep.pins;
            if pins.len() == 1{
                // ignore the check if non-shorting expose
                continue;
            }
            for pin in pins{
                for sp in &shorted_pins{
                    if sp.contains(pin){
                        let error_msg = format!(
                            "exposed shorted pins {:?} are also shorted with non-exposed pins {:?} which is not allowed",
                            pins,
                            sp
                        );
                        return quote::quote_spanned!{self.name.span()=>
                            compile_error!(#error_msg);
                        };
                        
                    }
                    
                }
            }
        }

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

        // TODO add a test to verify this
        let exposed_pin_type_check = {
            let instantiate_chip_vars = self.chip_map.iter().map(|(name, _)| {    
                let __name = quote::format_ident!("_{}",&name);
                quote! {let #__name = self.added_chip_map.get(#name).unwrap().get_pin_list();}
            });

            let checks = self.exposed_pins.iter().map(|ep|{
                if ep.pins.len() == 1{
                    // skip for single (non-shorted) exposed pin
                    return quote!{};
                }
                let zeroth_chip = &ep.pins[0].chip;
                let zeroth_pin = &ep.pins[0].pin;
                let zeroth_chip_ident = quote::format_ident!("_{}",zeroth_chip);
                let zeroth_extracted = quote!{
                    let first_type = &#zeroth_chip_ident.get(#zeroth_pin).unwrap().data_type;
                };
                let pin_checks = ep.pins.iter().map(|p|{
                    let _chip = &p.chip;
                    let _pin = &p.pin;
                    let chip_ident = quote::format_ident!("_{}",_chip);
                    quote!{
                        let md = #chip_ident.get(#_pin).unwrap();
                        if !matches!(md.pin_type,pcb_rs::PinType::Input){
                            return std::result::Result::Err(format!(
                                "chip {} pin {} is expected to be input type, as it is exposed and shorted with other pins, but was not. only input type pins are allowed to be shorted when exposing",
                                #_chip,#_pin
                            ));
                        }
                        if md.data_type != *first_type{
                            return std::result::Result::Err(format!(
                                "chip {} pin {} is expected to be of {} type, as it is shorted with pin of that type, but was found to be of {} type",
                                #_chip,#_pin,first_type,md.data_type
                            ));
                        }
                    }
                });
                quote!{
                    #zeroth_extracted
                    #(#pin_checks)*
                }
            });


            quote!{
                #(#instantiate_chip_vars)*
                #(#checks)*
            }
        };

        // ci is ChipInterface
        
        let ci_pin_map = self.exposed_pins.iter().map(|ep|{
            // note that metadata for all shorted exposed pins must be same, which
            // will be verified at building time, so we can just give metadata of first pin
            // and in case there is a single pin, it will be 0th
            let pin_name = &ep.pins[0].pin;
            let chip_name = &ep.pins[0].chip;
            let as_name = &ep.as_name;
            quote!{
                let __chip = self.chips.get(#chip_name).unwrap();
                let md = __chip.get_pin_list().get(#pin_name).unwrap().clone();
                ret.insert(#as_name,md);
            }
        });

        let ci_get_value = self.exposed_pins.iter().map(|ep|{
            // note that the shorted pins must be of input type, shorting multiple o/p
            // is not valid and will error at building time
            // thus, this is only valid when all pins are input type, and as all are shorted,
            // we can just give 0th pin's value
            let pin_name = &ep.pins[0].pin;
            let chip_name = &ep.pins[0].chip;
            let as_name = &ep.as_name;
            quote!{
                #as_name =>{
                    let __chip = self.chips.get(#chip_name).unwrap();
                    return __chip.get_pin_value(#pin_name);
                }
            }
        });

        let ci_set_value = self.exposed_pins.iter().map(|ep|{
            let t = ep.pins.iter().map(|cp|{
                let pin_name = &cp.pin;
                let chip_name = &cp.chip;
                quote!{
                    let __chip = self.chips.get_mut(#chip_name).unwrap();
                    if ! __chip.is_pin_tristated(#chip_name){
                        __chip.set_pin_value(#pin_name,val);
                    }
                }
            });
            let as_name = &ep.as_name;
            quote!{
                #as_name =>{
                    #(#t)*
                    return;
                }
            }
        });

        let ci_pin_tristated = self.exposed_pins.iter().map(|ep|{
            // as the only pins to be grouped in expose are of input type,
            // so it does not make sense to ask if the pins are tristated or not,
            // as that majorly matters when the pin is output type.
            // and when setting pins values, pins which are tristated are ignored anyways,
            // so we can just return false in that case
            let as_name = &ep.as_name;
            if ep.pins.len() == 1{
                let chip_name = &ep.pins[0].chip;
                let pin_name = &ep.pins[0].pin;
                quote!{
                    #as_name =>{
                        let __chip = self.chips.get(#chip_name).unwrap();
                        return __chip.is_pin_tristated(#pin_name);
                    }
                }
            }else{
                quote!{
                    #as_name =>{
                        false
                    }
                }
            }
            
        });

        let ci_pin_input_mode = self.exposed_pins.iter().map(|ep|{
            // as only input type pins are allowed to be shorted , in case there are shorted pins, 
            // they will have to be in input type
            let as_name = &ep.as_name;
            if ep.pins.len() == 1{
                let chip_name = &ep.pins[0].chip;
                let pin_name = &ep.pins[0].pin;
                quote!{
                    #as_name =>{
                        let __chip = self.chips.get(#chip_name).unwrap();
                        return __chip.in_input_mode(#pin_name);
                    }
                }
            }else{
                quote!{
                    #as_name =>{
                        return true;
                    }
                }
            }
            
        });

        quote! {
            
            pub struct #builder_name{
                added_chip_map:std::collections::HashMap<std::string::String,std::boxed::Box<dyn pcb_rs::HardwareModule>>,
                shorted_pins:std::vec::Vec<std::vec::Vec<pcb_rs::ChipPin>>,
                pin_metadata_cache:std::collections::HashMap<pcb_rs::ChipPin,pcb_rs::PinMetadata>
            }

            impl #builder_name{

                pub fn new()->Self{
                    let shorted = std::vec![#(#shorted_pins_tokens),*];
                    Self{
                        added_chip_map:std::collections::HashMap::new(),
                        shorted_pins:shorted,
                        pin_metadata_cache:std::collections::HashMap::new()
                    }
                }

                pub fn add_chip(mut self,name:&str,chip: std::boxed::Box<dyn pcb_rs::HardwareModule>)->Self{
                    self.added_chip_map.insert(name.to_string(),chip);
                    self
                }

                pub fn build(mut self)->std::result::Result<#pcb_name, std::string::String>{
                    self.check_added_all_chips()?;
                    self.check_valid_chips()?;
                    // this will validate pin connections as well as set up
                    // the pin metadata in hashmap
                    self.check_valid_pin_connection()?;
                    self.check_exposed_pin_types()?;
                    let pin_connections = self.get_pin_connections()?;

                    std::result::Result::Ok(#pcb_name{
                        chips:self.added_chip_map,
                        pin_connections
                    })
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

                // yes this does two things by also setting the chip metadata in hashmap, but otherwise there
                // would have been a lot of code duplication, so go with it for now
                fn check_valid_pin_connection(&mut self)->std::result::Result<(),std::string::String>{
                    #(#instantiate_chip_vars)*
                    #(#pin_connection_checks)*
                    
                    std::result::Result::Ok(())
                }

                fn check_exposed_pin_types(&self)->std::result::Result<(),std::string::String>{
                    #exposed_pin_type_check

                    std::result::Result::Ok(())
                }

                // This function can be optimized a bit by removing multiple iter() and map() calls
                // some of might be redundant
                fn get_pin_connections(&self)->std::result::Result<std::vec::Vec<pcb_rs::ConnectedPins>,std::string::String>{
                    use std::vec::Vec;
                    use pcb_rs::{ChipPin,PinType,ConnectedPins,PinMetadata};

                    let mut ret:Vec<ConnectedPins> = Vec::with_capacity(self.shorted_pins.len());
                    for group in &self.shorted_pins{
                        let input_pins = group.iter().filter(|pin|{
                            let md = self.pin_metadata_cache.get(pin).unwrap();
                            matches!(md.pin_type,pcb_rs::PinType::Input) || matches!(md.pin_type,pcb_rs::PinType::IO)
                        }).map(|pin|(*pin,self.pin_metadata_cache.get(pin).unwrap())).collect();

                        let output_pins = group.iter().filter(|pin|{
                            let md = self.pin_metadata_cache.get(pin).unwrap();
                            matches!(md.pin_type,pcb_rs::PinType::Output) || matches!(md.pin_type,pcb_rs::PinType::IO)
                        }).map(|pin|(*pin,self.pin_metadata_cache.get(pin).unwrap())).collect();
                        
                        ret.push(pcb_rs::get_pin_group(input_pins,output_pins)?);
                    }

                    Ok(ret)
                }

            }

            pub struct #pcb_name{
                chips:std::collections::HashMap<std::string::String,std::boxed::Box<dyn pcb_rs::HardwareModule>>,
                pin_connections:std::vec::Vec<pcb_rs::ConnectedPins>
            }

            impl #pcb_name{
                pub fn get_chip<'s,T:pcb_rs::HardwareModule>(&'s self,chip:&str)->std::option::Option<&'s T>{
                    match self.chips.get(chip){
                        std::option::Option::None => None,
                        Some(c)=>{
                            c.downcast_ref()
                        }
                    }
                }

                pub fn get_chip_mut<'s,T:pcb_rs::HardwareModule>(&'s mut self,chip:&str)->std::option::Option<&'s mut T>{
                    match self.chips.get_mut(chip){
                        std::option::Option::None => None,
                        Some(c)=>{
                            c.downcast_mut()
                        }
                    }
                }
            }

            impl pcb_rs::ChipInterface for #pcb_name{
                
                fn get_pin_list(&self) -> std::collections::HashMap<&'static str, pcb_rs::PinMetadata>{
                    let mut ret = std::collections::HashMap::new();
                    #(#ci_pin_map)*
                    ret
                }
                
                fn get_pin_value(&self, name: &str) -> std::option::Option<Box<dyn std::any::Any>>{
                    match name{
                        #(#ci_get_value),*
                        _ => None
                    }
                }
                
                fn set_pin_value(&mut self, name: &str, val: &dyn std::any::Any){
                    match name{
                        #(#ci_set_value),*
                        _ => {}
                    }
                }
                
                fn is_pin_tristated(&self, name: &str) -> bool{
                    match name{
                        #(#ci_pin_tristated),*
                        _ => false
                    }
                }
                
                fn in_input_mode(&self, name: &str) -> bool{
                    match name{
                        #(#ci_pin_input_mode),*
                        _ => false
                    }
                }
            }

            impl pcb_rs::Chip for #pcb_name{
                fn tick(&mut self){
                    use std::any::Any;
                    use std::option::Option;
                    use pcb_rs::{ChipPin,PinType,ConnectedPins,PinMetadata};


                    for chip in self.chips.values_mut(){
                        chip.tick();
                    }

                    for connection in &self.pin_connections{
                        match connection{
                            ConnectedPins::Pair{source,destination}=>{
                                // because we have made sure the chips and pins exist properly,
                                // we can unwrap directly
                                // also this is simplest, as there is a single input and single output pin,
                                // both of which are of respective types, so even if they're tristated,
                                //  their data types will match, and there won't be an issue
                                // TODO implement a test to verify this 
                                let src = self.chips.get(source.chip).unwrap();
                                let val = src.get_pin_value(source.pin).unwrap();
                                // we have to take it as ref, otherwise the box is passed around,
                                // instead of the data which we want
                                let data_ref = val.as_ref();
                                let dest = self.chips.get_mut(destination.chip).unwrap();
                                dest.set_pin_value(destination.pin,data_ref);
                            }
                            ConnectedPins::Broadcast{source,destinations}=>{
                                // now this can get tricky, as the source pin might be of type
                                // io, so it can be present in destinations as well, so we have to skip it
                                // as well as check that if there is any destination pin that is 
                                // io type, then it is set to input mode
                                // also we do not check if the source pin, if of io type
                                // is set to input mode or not, the destination pins will get 
                                // whatever its value is regardless
                                let chip = self.chips.get(source.chip).unwrap();
                                let val = chip.get_pin_value(source.pin).unwrap();
                                let data_ref = val.as_ref();
                                for dest in destinations{
                                    if dest == source{
                                        // accounts for the io type source pin
                                        continue;
                                    }
                                    let chip = self.chips.get_mut(dest.chip).unwrap();
                                    // we don't have to check if any other pin is of io type, because if it was
                                    // then taht set-up would be in the tristated group
                                    chip.set_pin_value(dest.pin,data_ref);
                                }
                            }
                            ConnectedPins::Tristated{sources,destinations}=>{
                                let mut val = Option::None;
                                let mut active_chip = ChipPin{
                                    chip: "unknown",
                                    pin: "unknown",
                                };
                                for src in sources{
                                    let chip = self.chips.get(src.chip).unwrap();
                                    // input mode check if specifically for io pins, which would be present in
                                    // both sources and destinations, and if one want to get the data in io pin
                                    // the pin must not be in tristated mode, but must be in input mode
                                    if !chip.in_input_mode(src.pin) && !chip.is_pin_tristated(src.pin){
                                        if val.is_some(){
                                            panic!("Multiple pins found active at the same time in a tristated group : pin {:?} and pin {:?} in group {:?}. Only one pin in a tristated group can be active at a time",src, active_chip,connection);
                                        }
                                        active_chip = *src;
                                        val = Option::Some(chip.get_pin_value(src.pin).unwrap());
                                    }
                                }
                                if let Some(val) = val{
                                    let data_ref = val.as_ref();
                                    for dest in destinations{
                                        // skip in case the pin is io type and present in both source and destinations
                                        if *dest == active_chip{
                                            continue;
                                        }
                                        let chip = self.chips.get_mut(dest.chip).unwrap();
                                        // skip tristated pins
                                        if chip.is_pin_tristated(dest.pin){
                                            continue;
                                        }
                                        chip.set_pin_value(dest.pin,data_ref);
                                    }
                                }
                                
                            }
                        }
                    }

                }
            }
        }
    }
}
