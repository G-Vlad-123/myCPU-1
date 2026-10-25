mod lexer;
mod tokens;
mod writer;

#[derive(Debug, snafu::Snafu)]
enum TopError {
    #[snafu(display("No file path was given."))]
    NoFilePathGiven,

    #[snafu(display("Couldn't open the file at {path} due to {source}.", path = path.display()))]
    CouldNotOpenFile {
        #[snafu(source)]
        source: std::io::Error,

        path: std::path::PathBuf,
    },

    #[snafu(display("{source}"))]
    #[snafu(context(false))]
    LexerError {
        #[snafu(source(from(exact)))]
        source: lexer::LexerError,
    },

    #[snafu(display("Couldn't acces the current active directory due to {source}."))]
    InaccsesableCurrentDirectory {
        #[snafu(source)]
        source: std::io::Error,
    },

    #[snafu(display("Couldn't create/open a new file at {path} due to {source}.", path = path.display()))]
    CouldNotCreateFile {
        #[snafu(source)]
        source: std::io::Error,

        path: std::path::PathBuf,
    },

    #[snafu(display("{source}"))]
    #[snafu(context(false))]
    WritingError {
        #[snafu(source(from(exact)))]
        source: writer::WritingError,
    },
}

fn main() -> Result<(), TopError> {
    use snafu::prelude::*;
    use tap::prelude::*;

    let path = std::env::args()
        .nth(0)
        .context(NoFilePathGivenSnafu)?
        .as_str()
        .pipe(AsRef::<std::path::Path>::as_ref)
        .to_path_buf();

    let mut file_name = "".as_ref();

    let program = path
        .as_path()
        .tap(|path| file_name = path.file_prefix().unwrap_or("placeholder_name".as_ref()))
        .pipe(std::fs::File::open)
        .with_context(|_| CouldNotOpenFileSnafu { path: path.clone() })?
        .pipe(lexer::tokenize_file)?;

    let file_name = file_name.to_owned().tap_mut(|name| name.push(".bin"));

    let path = std::env::current_dir()
        .context(InaccsesableCurrentDirectorySnafu)?
        .tap_mut(|path| path.set_file_name(file_name));

    let mut file = std::fs::File::create(&path).context(CouldNotCreateFileSnafu { path })?;

    writer::write(file, program.iter()).map_err(Into::into)
}
