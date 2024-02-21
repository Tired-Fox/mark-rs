extern crate mark_rs;

use crossterm::{terminal::size, cursor::position};

fn main() -> std::io::Result<()> {
    println!("{:?}", size());
    println!("{:?}", position());
    Ok(())
}
