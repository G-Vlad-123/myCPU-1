use crate::tokens::values::{Address, Value, Word};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
struct HardcodedAddress {
    high: Word,
    low: Word,
}

#[derive(Debug)]
pub struct Context {
    address_names: HashMap<String, HardcodedAddress>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            address_names: HashMap::new(),
        }
    }

    pub fn add_address_name(&mut self, address_name: String, high: Word, low: Word) {
        self.address_names
            .insert(address_name, HardcodedAddress { high, low });
    }
}

pub trait AppendData {
    type Error;

    fn append_data(self, ctx: &Context, data: &mut Vec<Word>) -> Result<(), Self::Error>;
}

impl<T: Into<Word>> AppendData for T {
    type Error = !;

    fn append_data(self, _: &Context, data: &mut Vec<Word>) -> Result<(), !> {
        data.push(self.into());
        Ok(())
    }
}

impl AppendData for Value {
    type Error = !;

    fn append_data(self, ctx: &Context, data: &mut Vec<Word>) -> Result<(), !> {
        match self {
            Self::Literal(literal) => {
                data.push(Word::WF);
                literal.append_data(ctx, data);
                Ok(())
            }
            Self::Register(register) => register.append_data(ctx, data),
        }
    }
}

impl AppendData for HardcodedAddress {
    type Error = !;

    fn append_data(self, ctx: &Context, data: &mut Vec<Word>) -> Result<(), !> {
        self.high.append_data(ctx, data);
        self.low.append_data(ctx, data);
        Ok(())
    }
}

#[derive(Debug, snafu::Snafu)]
pub struct InvalidAddressName {
    name: String,
}

impl AppendData for Address {
    type Error = InvalidAddressName;

    fn append_data(self, ctx: &Context, data: &mut Vec<Word>) -> Result<(), InvalidAddressName> {
        match self {
            Address::Hardcoded { high, low } => {
                high.append_data(ctx, data);
                low.append_data(ctx, data);
                Ok(())
            }
            Address::Named { name } => {
                let Some(addr) = ctx.address_names.get(name.as_str()) else {
                    return Err(InvalidAddressName { name });
                };

                (*addr).append_data(ctx, data);
                Ok(())
            }
        }
    }
}
