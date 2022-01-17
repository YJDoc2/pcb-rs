// This is a temp file to write how I want the final version to look like

pub enum MemState {
    Active,
    Inactive,
}

pub enum MemMode {
    Read,
    Write,
}

#[derive(Chip, Default)]
pub struct Processor {
    #[pin(type = output)]
    address_bus: u8,
    #[pin(type = input)]
    intr: bool,
    #[pin(type = io)]
    data_bus: u8,
    #[pin(type = output)]
    mem_state: MemState,
    #[pin(type = output)]
    mem_mode: MemMode,

    instr_cache: Vec<u8>,
    AX: u8,
    BX: u8,
    IP: u8,
}

impl HWModule for Processor {
    fn tick(&mut self) {
        // code to read instruction from cache, if
        // seg fault issue /  incomplete instruction,
        // set to read more instruction from mem
    }
}

#[derive(Chip, Default)]
pub struct Memory {
    #[pin(type = input)]
    op_mode: MemMode,
    #[pin(type = input)]
    address: u8,
    #[pin(type = io)]
    data: u8,
    #[pin(type = input)]
    active: MemState,

    mem: [u8; 255],
}

impl HWModule for Memory {
    fn tick(&mut self) {
        // code to see if the state pin is set to state active
        // and the take appropriate action according to mode
    }
}

pcb!(BasicComputer{
    module processor;
    module memory;

    processor::address_bus - memory::address
    processor::data_bus - memory::data
    processor::mem_mode - memory::op_mode
    processor::mem_state - memory::active

    expose memory::address
    expose memory::data
    expose memory::op_mode
    expose memory::active
});

fn get_basic_computer() -> BasicComputer {
    let memory = Box::new(Memory::default());
    let processor = Box::new(Processor::default());
    BasicComputerBuilder::default()
        .add_module("processor", processor)
        .add_module("memory", memory)
        .build()
}

fn main() {
    let mut basic_computer = get_basic_computer();
    {
        let processor: &mut Processor = basic_computer.get_module("processor").unwrap();
        // do something, maybe manually set IP etc
    }
    {
        let memory: &mut Memory = basic_computer.get_module("memory").unwrap();
        // maybe set data in the memory manually
    }
    loop {
        basic_computer.tick();
    }
}
