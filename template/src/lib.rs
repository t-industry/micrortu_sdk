use micrortu_sdk::{BlockPorts, FactoryInput, Shared, StepResult, params, ports, register_block};
use static_cell::StaticCell;

pub struct Counter;

ports! {
    #[block_names(counter)]
    pub struct Ports {
      count: TI13 InOut 1 1,
    }
}
params! {
    #[block_names(counter)]
    pub struct Params {}
}

pub fn factory(_: &FactoryInput) -> Option<&'static mut Counter> {
    static COUTNER: StaticCell<Counter> = StaticCell::new();
    Some(COUTNER.init(Counter))
}

pub fn init(_: &mut Shared, _: &mut Counter) -> StepResult {
    0
}

pub fn step(shared: &mut Shared, _: &mut Counter) -> StepResult {
    let ports = Ports::parse(&mut shared.latched_ports[..]);

    ports.count.value += 1.;

    0
}

register_block!(Counter, counter, factory, init, step);
