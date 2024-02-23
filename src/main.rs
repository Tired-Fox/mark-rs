extern crate mark_rs;

use mark_rs::style::{color, Style};

fn main() {
    let br = Style::builder()
        .fg(color!(red))
        .bold();
    let ib = Style::builder()
        .fg(color!(blue))
        .italic();

    // \x1b[1;32mHello\x1b[22;39m, \x1b[3;34mworld!\x1b[0m
    println!("{br}Hello{br:-}, {ib}world!{ib:-}");
}
