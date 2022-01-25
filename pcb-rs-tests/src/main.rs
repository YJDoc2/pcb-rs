use pcb_rs::pcb;

pcb!(BasicComputer {
    chip processor;
    chip memory;
    chip dma;

    processor::address_bus - memory::address;
    memory::address - dma::address_bus;
    processor::data_bus - memory::data;
    processor::mem_mode - memory::op_mode;
    processor::mem_state - memory::active;

    expose memory::address;
    expose memory::data;
    expose memory::op_mode;
    expose memory::active;
});

fn main() {}
