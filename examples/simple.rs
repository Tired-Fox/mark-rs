use std::io::{stdout, Write};

use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor, SetBackgroundColor},
    ExecutableCommand,
    event
};

fn main() -> std::io::Result<()> {
    execute!(
        stdout(),
        SetBackgroundColor(Color::Black),
        SetForegroundColor(Color::AnsiValue(207)),
        Print("Hello"),
        ResetColor,
    )?;

    stdout()
        .execute(SetForegroundColor(Color::Rgb { r: 220, g: 100, b: 50 }))?
        .execute(SetBackgroundColor(Color::White))?
        .execute(Print("World"))?
        .execute(ResetColor)?;

    Ok(())
}
