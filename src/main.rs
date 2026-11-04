mod bit;
mod cpu;
mod manipulation;
mod output;
mod storage;

use tap::TapFallible;

enum TopError {
    UnableToOpenData(std::io::Error),
    CpuError(cpu::CpuError),
}

impl std::fmt::Debug for TopError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TopError::UnableToOpenData(error) => write!(f, "{error}"),
            TopError::CpuError(error) => write!(f, "{error}"),
        }
    }
}

fn main() -> Result<(), TopError> {
    let mut args = std::env::args().skip(1);

    let mut data = storage::data::Data::from_file(args.next().expect("No file provided"))
        .map_err(TopError::UnableToOpenData)?;

    let mut cpu = cpu::CPU::new(args.map(|arg| {
        use bit::{One as O, Zero as Z};
        let word = storage::Word::new;

        match arg.as_str().trim() {
            "0" => word([Z, Z, Z, Z]),
            "1" => word([Z, Z, Z, O]),
            "2" => word([Z, Z, O, Z]),
            "3" => word([Z, Z, O, O]),
            "4" => word([Z, O, Z, Z]),
            "5" => word([Z, O, Z, O]),
            "6" => word([Z, O, O, Z]),
            "7" => word([Z, O, O, O]),
            "8" => word([O, Z, Z, Z]),
            "9" => word([O, Z, Z, O]),
            "A" => word([O, Z, O, Z]),
            "B" => word([O, Z, O, O]),
            "C" => word([O, O, Z, Z]),
            "D" => word([O, O, Z, O]),
            "E" => word([O, O, O, Z]),
            "F" => word([O, O, O, O]),
            _ => panic!("Invalid argument `{arg}` given!"),
        }
    }));

    cpu.run(&mut data)
        .tap_err(|err| println!("A fatal error occired: {err:?}"))
        .map_err(TopError::CpuError)?;

    output::output(data);

    Ok(())
}
