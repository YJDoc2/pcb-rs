use pcb_rs::{pcb, Chip};

#[derive(Chip, Default)]
struct TestChip1 {
    #[pin(input)]
    pin1: u8,
    #[pin(output)]
    pin2: bool,
    #[pin(output)]
    pin3: String,

    internal_1: String,
    internal_2: u8,
}

impl Chip for TestChip1 {
    fn tick(&mut self) {}
}

#[derive(Chip, Default)]
struct TestChip2 {
    #[pin(output)]
    pin1: u8,
    #[pin(input)]
    pin2: bool,
    #[pin(input)]
    pin3: String,

    internal_1: String,
    internal_2: bool,
}

impl Chip for TestChip2 {
    fn tick(&mut self) {}
}

pcb!(TestPCB{
    chip tc1;
    chip tc2;

    tc1::pin1 - tc2::pin1;
    tc1::pin2 - tc2::pin2;
    tc1::pin3 - tc2::pin3;

});
fn main() {
    let tc1 = Box::new(TestChip1::default());
    let tc2 = Box::new(TestChip2::default());

    let temp = TestPCBBuilder::new()
        .add_chip("tc1", tc1)
        .add_chip("tc2", tc2);
    let mut test_pcb = temp.build().unwrap();
    for _ in 0..8 {
        test_pcb.tick();
    }
}
