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
    let mut cpu = cpu::CPU::new();

    let mut data =
        storage::data::Data::from_file(std::env::args().next().expect("No file provided"))
            .map_err(TopError::UnableToOpenData)?;

    cpu.run(&mut data)
        .tap_err(|err| println!("A fatal error occired: {err:?}"))
        .map_err(TopError::CpuError)?;

    output::output(data);

    Ok(())
}
