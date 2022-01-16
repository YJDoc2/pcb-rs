#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum PinType {
    Input,
    Output,
    IO,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct PinInfo<'s> {
    name: &'s str,
    pin_type: PinType,
}

impl<'s> PinInfo<'s> {
    pub fn new(name: &'s str, t: PinType) -> Self {
        PinInfo {
            name: name,
            pin_type: t,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct HWPinValue<'s> {
    pub name: &'s str,
    pub value: bool,
}

impl<'s> HWPinValue<'s> {
    pub fn new(name: &'s str, v: bool) -> Self {
        HWPinValue {
            name: name,
            value: v,
        }
    }
}
