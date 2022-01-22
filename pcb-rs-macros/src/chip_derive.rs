use pcb_rs_traits::PinType;
use proc_macro2::TokenStream;
use quote::quote;

const PIN_ATTRIBUTE: &str = "pin";

const INVALID_PIN_ATTR_ERR: &str =
    "invalid pin attribute, currently only #[pin(input|output|io)] is supported";

const PIN_TYPE_INPUT: &str = "input";
const PIN_TYPE_OUTPUT: &str = "output";
const PIN_TYPE_IO: &str = "io";

#[derive(Debug)]
struct __PinMetadata<'a> {
    name: &'a syn::Ident,
    pin_type: PinType,
    data_type: &'a syn::Type,
}

fn get_pin_attr(f: &syn::Field) -> &syn::Attribute {
    for attr in &f.attrs {
        if attr.path.segments.len() == 1 && attr.path.segments[0].ident == PIN_ATTRIBUTE {
            return attr;
        }
    }
    // as we have already filtered fields to which have attributes, and
    // fields which have attribute other than pin, will give compile-error from rust's side
    // not finding the attribute is essentially unreachable
    unreachable!()
}

fn get_compiler_error<T, U>(t: T, m: U) -> TokenStream
where
    T: quote::ToTokens,
    U: std::fmt::Display,
{
    syn::Error::new_spanned(t, m).to_compile_error()
}

fn get_pin_type(nm: syn::NestedMeta) -> Result<PinType, TokenStream> {
    match nm {
        syn::NestedMeta::Meta(syn::Meta::Path(mut path)) => {
            if path.segments.len() != 1 {
                return Err(get_compiler_error(path, INVALID_PIN_ATTR_ERR));
            }
            let ptype = path.segments.pop().unwrap().into_value().ident;
            match ptype.to_string().as_str() {
                PIN_TYPE_INPUT => Ok(PinType::Input),
                PIN_TYPE_OUTPUT => Ok(PinType::Output),
                PIN_TYPE_IO => Ok(PinType::IO),
                _ => return Err(get_compiler_error(ptype, INVALID_PIN_ATTR_ERR)),
            }
        }
        meta => return Err(get_compiler_error(meta, INVALID_PIN_ATTR_ERR)),
    }
}

fn get_pin_metadata<'a>(fields: &'a [&syn::Field]) -> Result<Vec<__PinMetadata<'a>>, TokenStream> {
    let mut ret = Vec::with_capacity(fields.len());
    for field in fields {
        let pin_attr = get_pin_attr(field);
        match pin_attr.parse_meta() {
            Err(e) => return Err(e.to_compile_error()),
            Ok(syn::Meta::List(mut args)) => {
                let pin_type = get_pin_type(args.nested.pop().unwrap().into_value())?;
                ret.push(__PinMetadata {
                    name: &field.ident.as_ref().unwrap(),
                    pin_type: pin_type,
                    data_type: &field.ty,
                })
            }
            Ok(meta) => return Err(get_compiler_error(meta, INVALID_PIN_ATTR_ERR)),
        }
    }
    Ok(ret)
}

pub fn derive_chip_impl(name: &syn::Ident, data: &syn::DataStruct) -> TokenStream {
    let fields = match &data.fields {
        syn::Fields::Unit | syn::Fields::Unnamed(_) => {
            panic!("Chip derive is only supported for named field structs")
        }
        syn::Fields::Named(f) => &f.named,
    };

    let pin_fields = {
        let mut ret = Vec::with_capacity(fields.len());
        for field in fields {
            if field.attrs.len() != 0 {
                ret.push(field);
            }
        }
        ret
    };
    let metadata = match get_pin_metadata(&pin_fields) {
        Result::Ok(md) => md,
        Result::Err(e) => return e,
    };

    let pin_hashmap_arm = metadata.iter().map(|p| {
        let name = p.name.to_string();
        let ptype = syn::Ident::new(&p.pin_type.to_string(), data.struct_token.span);
        // have to do that, as we can't access it as #p.data_type
        let __temp = p.data_type;
        let dtype = quote! {#__temp}.to_string();
        quote! {
            #name, pcb_rs::PinMetadata{
                pin_type:pcb_rs::PinType::#ptype,
                data_type:#dtype
            }
        }
    });

    let get_pin_match_arm = metadata.iter().map(|p| {
        let name = p.name;
        let name_string = p.name.to_string();
        quote! {
            #name_string => std::option::Option::Some(std::boxed::Box::new(self.#name.clone()))

        }
    });

    let set_pin_match_arm = metadata.iter().map(|p| {
        let __name = p.name;
        let name_string = p.name.to_string();
        let dtype = p.data_type;
        let assertion_err_msg = format!("internal error in pcb_rs derive chip : value sent to chip {} pin {} is of incorrect type",name,__name);
        quote! {
            #name_string =>{
                assert!(val.is::<#dtype>(), #assertion_err_msg);
                self.#__name = val.downcast_ref::<#dtype>().unwrap().clone();
            }
        }
    });

    quote! {
        impl pcb_rs::ChipInterface for #name{

            fn get_pin_list(&self) -> std::collections::HashMap<&'static str, pcb_rs::PinMetadata>{
                use std::collections::HashMap;
                let mut ret = HashMap::new();
                #(ret.insert(#pin_hashmap_arm );)*
                ret
            }


            fn get_pin_value(&self,name: &str) -> std::option::Option<Box<dyn std::any::Any>>{
                use std::any::Any;
                use std::boxed::Box;
                match name{
                    #(#get_pin_match_arm,)*
                    _ => None
                }
            }


            fn set_pin_value(&mut self,name: &str, val: &dyn std::any::Any){
                use std::any::Any;
                match name{
                    #(#set_pin_match_arm,)*
                    _ => {}
                }
            }

        }
    }
}
