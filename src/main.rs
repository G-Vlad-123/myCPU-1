mod bit;
mod cpu;
mod storage;

fn main() {
    let mut cpu = cpu::CPU::new();

    let mut data = storage::data::Data::new();

    data.set_word(0, storage::word::Word::FILLED);
    println!("{:?}", data.get_word(0))
}
