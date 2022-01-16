use super::pin::{HWPinValue, PinInfo, PinType};

pub trait HWChip {
    fn get_pin_list(&self) -> Vec<PinInfo>;
    fn run(&mut self, inputs: &[HWPinValue]) -> Vec<HWPinValue>;
}

macro_rules! make_simple_chip {
    ($name:ident,$op:tt) => {
        pub struct $name {}

        impl $name {
            pub fn new() -> Self {
                $name {}
            }
        }

        impl HWChip for $name {
            fn get_pin_list(&self) -> Vec<PinInfo> {
                vec![
                    PinInfo::new("In1", PinType::Input),
                    PinInfo::new("In2", PinType::Input),
                    PinInfo::new("Out1", PinType::Output),
                ]
            }

            fn run(&mut self, inputs: &[HWPinValue]) -> Vec<HWPinValue> {
                let mut i1 = false;
                let mut i2 = false;
                for i in inputs {
                    match i.name {
                        "In1" => i1 = i.value,
                        "In2" => i2 = i.value,
                        _ => {}
                    }
                }
                vec![HWPinValue::new("Out1", i1 $op i2)]
            }
        }
    };
}

make_simple_chip!(AndChip,&);

make_simple_chip!(OrChip,|);

make_simple_chip!(XorChip,^);

pub struct NotChip {}

impl NotChip {
    pub fn new() -> Self {
        NotChip {}
    }
}

impl HWChip for NotChip {
    fn get_pin_list(&self) -> Vec<PinInfo> {
        vec![
            PinInfo::new("In1", PinType::Input),
            PinInfo::new("Out1", PinType::Output),
        ]
    }

    fn run(&mut self, inputs: &[HWPinValue]) -> Vec<HWPinValue> {
        let mut i1 = false;
        for i in inputs {
            match i.name {
                "In1" => i1 = i.value,
                _ => {}
            }
        }
        vec![HWPinValue::new("Out1", !i1)]
    }
}

macro_rules! make_negated_chips {
    ($name:ident,$op:tt) => {
        pub struct $name {}

        impl $name {
            pub fn new() -> Self {
                $name {}
            }
        }

        impl HWChip for $name {
            fn get_pin_list(&self) -> Vec<PinInfo> {
                vec![
                    PinInfo::new("In1", PinType::Input),
                    PinInfo::new("In2", PinType::Input),
                    PinInfo::new("Out1", PinType::Output),
                ]
            }

            fn run(&mut self, inputs: &[HWPinValue]) -> Vec<HWPinValue> {
                let mut i1 = false;
                let mut i2 = false;
                for i in inputs {
                    match i.name {
                        "In1" => i1 = i.value,
                        "In2" => i2 = i.value,
                        _ => {}
                    }
                }
                vec![HWPinValue::new("Out1", !(i1 $op i2))]
            }
        }
    };
}

make_negated_chips!(NandChip,&);

make_negated_chips!(NorChip,|);

make_negated_chips!(XnorChip,^);

macro_rules! simple_chip_test {
    ($name:ident,$chip_name:ident,$op:tt) => {
        #[test]
        fn $name () {
            let mut c = $chip_name::new();
            for i in [true, false] {
                for j in [true, false] {
                    let v = c.run(&[HWPinValue::new("In1", i), HWPinValue::new("In2", j)]);
                    assert_eq!(v.len(), 1);
                    assert_eq!(v[0].name, "Out1");
                    assert_eq!(v[0].value, i $op j);
                }
            }
        }
    };
}

macro_rules! simple_negation_chip_test {
    ($name:ident,$chip_name:ident,$op:tt) => {
        #[test]
        fn $name () {
            let mut c = $chip_name::new();
            for i in [true, false] {
                for j in [true, false] {
                    let v = c.run(&[HWPinValue::new("In1", i), HWPinValue::new("In2", j)]);
                    assert_eq!(v.len(), 1);
                    assert_eq!(v[0].name, "Out1");
                    assert_eq!(v[0].value, !(i $op j));
                }
            }
        }
    };
}

simple_chip_test!(test_and_chip,AndChip,&);
simple_chip_test!(test_or_chip,OrChip,|);
simple_chip_test!(test_xor_chip,XorChip,^);

simple_negation_chip_test!(test_nand_chip,NandChip,&);
simple_negation_chip_test!(test_nor_chip,NorChip,|);
simple_negation_chip_test!(test_xnor_chip,XnorChip,^);

#[test]
fn test_not_chip() {
    let mut c = NotChip::new();
    for i in [true, false] {
        let v = c.run(&[HWPinValue::new("In1", i)]);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].name, "Out1");
        assert_eq!(v[0].value, !i);
    }
}
