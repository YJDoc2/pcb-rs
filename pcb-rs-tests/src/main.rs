use pcb_rs::{Chip, ChipInterface};

#[derive(Clone, Copy)]
pub enum MemState {
    Active,
    Inactive,
}

impl Default for MemState {
    fn default() -> Self {
        MemState::Inactive
    }
}

#[derive(Clone, Copy)]
pub enum MemMode {
    Read,
    Write,
}

impl Default for MemMode {
    fn default() -> Self {
        MemMode::Read
    }
}

#[derive(Chip, Default)]
pub struct Processor {
    #[pin(output)]
    address_bus: u8,
    #[pin(input)]
    intr: bool,
    #[pin(io)]
    data_bus: u8,
    #[pin(output)]
    mem_state: MemState,
    #[pin(output)]
    mem_mode: MemMode,

    instr_cache: Vec<u8>,
    AX: u8,
    BX: u8,
    IP: u8,
}

fn main() {
    let mut p = Box::new(Processor::default());
    let _p = p.as_mut() as &mut dyn ChipInterface;
    println!("{:#?}", p.get_pin_list());
    println!(
        "{:?}",
        p.get_pin_value("data_bus").unwrap().downcast_ref::<u8>()
    );
    p.set_pin_value("data_bus", &5_u8);
    println!(
        "{:?}",
        p.get_pin_value("data_bus").unwrap().downcast_ref::<u8>()
    );
}
