extern crate format;

use format::format;

fn main() {
    println!("Hello, world!");
    format!("Hello, {}!", "world", world="Hello");
}