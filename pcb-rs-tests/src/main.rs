use pcb_rs::pcb;

pcb!(BasicComputer {
    chip processor;
    chip memory;

    processor::address_bus - memory::address;

    expose processor::address_bus;
});

fn main() {}
