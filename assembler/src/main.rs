mod lexer;
mod tokens;
mod writer;

fn main() -> Result<(), BigErrorThatEatsEverything> {
    let file_path = std::env::args().nth(0).ok_or(ArgumentNotGivenError)?;
    let program: tokens::Program = lexer::tokenize_file(&file_path)?;
    writer::write_new_file(program)?;
}
