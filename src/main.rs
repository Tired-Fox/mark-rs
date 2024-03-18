extern crate mark_rs;

use mark_rs::style::{color, Color, Style};

fn main() {
    let br = Style::builder()
        .fg(color!(0%, 73%, 78%, 8%))
        .bold();
    let ib = Style::builder()
        .fg(Color::BLUE)
        .italic();

    // \x1b[1;32mHello\x1b[22;39m, \x1b[3;34mworld!\x1b[0m
    println!("{br}Hello{br:-}, {ib}world!{ib:-}");
}
