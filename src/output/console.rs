use std::fmt;
use std::io::Write;
use termcolor::{Color, ColorSpec, StandardStream, WriteColor};

use crate::{lexer::SourceMap, FileContext};

pub struct Console {}

impl Console {
    pub fn emit(
        level: Level,
        file_context: &FileContext,
        prefix: &str,
        message: &str,
        index: u32,
        len: u32,
    ) -> std::io::Result<()> {
        if level != Level::Bug {
        } else {
            let mut stream = StandardStream::stdout(termcolor::ColorChoice::Auto);

            let mut message_color = level.color();
            message_color.set_bold(true);

            let mut white_color = ColorSpec::new();
            white_color.set_fg(Some(PLAIN_WHITE));
            white_color.set_bold(true);

            let mut prompt_color = ColorSpec::new();
            prompt_color.set_fg(Some(PROMPT_COLOR));
            prompt_color.set_intense(true);
            prompt_color.set_bold(true);

            stream.set_color(&message_color)?;

            write!(stream, "{}", level)?;

            stream.set_color(&white_color)?;

            writeln!(stream, ": {}: {}", prefix, message)?;
        }

        Ok(())
    }
}
