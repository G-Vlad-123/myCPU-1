mod bit;
mod cpu;
mod manipulation;
mod storage;

use tap::TapFallible;

fn main() {
    let mut cpu = cpu::CPU::new();

    let mut data = storage::data::Data::new();

    let _ = cpu
        .run(&mut data)
        .tap_err(|err| println!("A fatal error occired: {err:?}"));
}
