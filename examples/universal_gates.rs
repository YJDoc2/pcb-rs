use pcb_rs::chip::*;
use pcb_rs::pin::*;

fn main() {
    test_nand_is_universal();
}

struct NandNot {
    nand: NandChip,
}
impl NandNot {
    fn new() -> Self {
        Self {
            nand: NandChip::new(),
        }
    }
}
impl HWChip for NandNot {
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
        self.nand
            .run(&[HWPinValue::new("In1", i1), HWPinValue::new("In2", i1)])
    }
}

struct NandOr {
    nand1: NandNot,
    nand2: NandNot,
    nand3: NandChip,
}

impl NandOr {
    fn new() -> Self {
        Self {
            nand1: NandNot::new(),
            nand2: NandNot::new(),
            nand3: NandChip::new(),
        }
    }
}

impl HWChip for NandOr {
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
        let t1 = self.nand1.run(&[HWPinValue::new("In1", i1)]);
        let t2 = self.nand2.run(&[HWPinValue::new("In1", i2)]);
        self.nand3.run(&[
            HWPinValue::new("In1", t1[0].value),
            HWPinValue::new("In2", t2[0].value),
        ])
    }
}

struct NandAnd {
    nand1: NandNot,
    nand2: NandChip,
}

impl NandAnd {
    fn new() -> Self {
        Self {
            nand1: NandNot::new(),
            nand2: NandChip::new(),
        }
    }
}

impl HWChip for NandAnd {
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
        let t = self
            .nand2
            .run(&[HWPinValue::new("In1", i1), HWPinValue::new("In2", i2)]);
        self.nand1.run(&[HWPinValue::new("In1", t[0].value)])
    }
}

fn test_nand_is_universal() {
    let mut nand_not = NandNot::new();
    let mut not = NotChip::new();
    for i in [true, false] {
        let nand_out = nand_not.run(&[HWPinValue::new("In1", i)]);
        let not_out = not.run(&[HWPinValue::new("In1", i)]);
        assert_eq!(nand_out, not_out);
    }
    let mut or = OrChip::new();
    let mut and = AndChip::new();
    let mut nand_or = NandOr::new();
    let mut nand_and = NandAnd::new();
    for i in [true, false] {
        for j in [true, false] {
            let t = [HWPinValue::new("In1", i), HWPinValue::new("In2", j)];
            assert_eq!(or.run(&t), nand_or.run(&t));
            assert_eq!(and.run(&t), nand_and.run(&t));
        }
    }
}
