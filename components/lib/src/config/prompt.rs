//! Interactive terminal prompting utilities for reading configuration.

use std::io::{self, IsTerminal, Write, stdin};

/// Whether interactive prompting is available. Only `carddav config` prompts;
/// the runtime commands never do.
pub fn prompt_enabled() -> bool {
    io::stdin().is_terminal()
}

pub trait PromptIO {
    fn read_line_trimmed(&mut self, prompt: &str) -> io::Result<String>;
    fn read_password(&mut self, prompt: &str) -> io::Result<String>;
    /// Print a section heading to the prompt sink (stderr). Default writes to
    /// stderr; does not consume any stdin input.
    fn heading(&mut self, text: &str) -> io::Result<()> {
        eprint!("{text}");
        io::stderr().flush()
    }
}

pub struct SystemPromptIO;

impl PromptIO for SystemPromptIO {
    fn read_line_trimmed(&mut self, prompt: &str) -> io::Result<String> {
        eprint!("{prompt}");
        io::stderr().flush()?;
        let mut line = String::new();
        stdin().read_line(&mut line)?;
        Ok(line.trim().to_string())
    }

    fn read_password(&mut self, prompt: &str) -> io::Result<String> {
        eprint!("{prompt}");
        io::stderr().flush()?;

        let config = if io::stdin().is_terminal() {
            rpassword::ConfigBuilder::new().build()
        } else {
            rpassword::ConfigBuilder::new()
                .input_reader(io::stdin())
                .output_discard()
                .build()
        };
        rpassword::read_password_with_config(config)
    }
}

/// Prompt for a string with an optional current value shown as the default.
/// An empty input keeps the current value, or yields `None` when there is none
/// (the caller rejects `None` for required fields).
pub fn prompt_optional(
    io: &mut dyn PromptIO,
    prompt: &str,
    current: Option<&str>,
) -> io::Result<Option<String>> {
    let shown = match current {
        Some(value) => format!(" [{value}]"),
        None => String::new(),
    };
    let line = io.read_line_trimmed(&format!("{prompt}{shown}: "))?;
    if line.is_empty() {
        Ok(current.map(str::to_string))
    } else {
        Ok(Some(line))
    }
}

/// Prompt for a hidden password. The current value is shown only as a masked
/// `[keep existing]` placeholder; an empty input keeps it (or yields `None`
/// when absent, which the caller rejects for required passwords).
pub fn prompt_password(
    io: &mut dyn PromptIO,
    prompt: &str,
    current: Option<&str>,
) -> io::Result<Option<String>> {
    let shown = if current.is_some() {
        " [keep existing]"
    } else {
        ""
    };
    let line = io.read_password(&format!("{prompt}{shown}: "))?;
    if line.is_empty() {
        Ok(current.map(str::to_string))
    } else {
        Ok(Some(line))
    }
}

/// Prompt for a yes/no boolean; an empty input keeps the shown default.
/// Anything other than `y`/`yes`/`n`/`no` re-prompts.
pub fn prompt_yn(io: &mut dyn PromptIO, prompt: &str, current: bool) -> io::Result<bool> {
    let shown = if current { "[y]" } else { "[n]" };
    loop {
        let line = io.read_line_trimmed(&format!("{prompt} {shown}: "))?;
        match line.to_ascii_lowercase().as_str() {
            "" => return Ok(current),
            "y" | "yes" => return Ok(true),
            "n" | "no" => return Ok(false),
            _ => {
                io::stderr().write_all(b"Please answer y or n.\n")?;
                io::stderr().flush()?;
            }
        }
    }
}
