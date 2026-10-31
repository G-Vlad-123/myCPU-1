use crate::bit;
use crate::storage::{Word, data};
use colored::Colorize;

fn get_colour(word: Word) -> colored::ColoredString {
    let [shaded, red, green, blue] = word.into_bits();

    let red = if red.is_one() { 255 } else { 0 };
    let green = if green.is_one() { 255 } else { 0 };
    let blue = if blue.is_one() { 255 } else { 0 };

    let text = if shaded.is_zero() { "■" } else { "▤" };

    text.truecolor(red, green, blue)
}

pub fn output(final_data: data::Data) {
    let display = std::fmt::from_fn(|f| {
        let mut address = data::Address::new(
            Word::new([bit::One, bit::Zero, bit::One, bit::One]),
            Word::new([bit::One; 4]),
        );

        let mut counter = 8;

        while address.increment().0.is_zero() {
            write!(f, "{}", get_colour(final_data.get_word(address)))?;

            counter -= 1;
            if counter == 0 {
                counter = 8;
                write!(f, "\n")?;
            }
        }

        Ok(())
    });

    println!("{display}")
}
