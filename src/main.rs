extern crate mark_rs;

use mark_rs::{
    style::{
        color,
        flags::*,
        Hyperlink,
        Style,
    },
    terminal::Capabilities,
};

fn main() -> std::io::Result<()> {
    /*
        Formatting:
            The formatting of text revolves around using Rust's built-in formatting. The library provides
            a AnsiSequence Trait that allows for the ansi sequence code, this is without `\x1b[` and `m`,
            to be retrieved along with what code is needed to reset the styling.

            Calling `to_string` on most of the libraries provided objects results in the full ansi
            sequence; Ex: {style} == `\x1b[32m`. Add `:-` in the format to reset the styling:
            `"{style:-}"` == `\x1b[39m`. If a style has an alternate form like color: defaults to foreground
            and the alternate is background, then use `:#` to use the alternate form. Ex: `{style:#}` == `\x1b[42m`.
            This also works with the reset so `{style:-#}` == `\x1b[49m`.

            Most of the structs and enums provided by the library implement the `Display` trait with formatting.
            Each struct also implements the `AnsiSequence` trait allowing for individual ansi codes and
            reset codes to be retrieved. Each struct and enums also implement `Clone` and `Copy` to make
            it easier to create a custom styling framework. Hyperlink is only `Clone` or `Copy` if whatever
            it wraps is also `Clone` or `Copy`. Since `Style` contains a `Hyperlink` this also applies
            to `Style`.
    */

    // Color format supports hsl, hsv, cymk, rgb, and hex, system colors, and xterm colors
    let hsl = color!(hsl 217 69% 68%);
    let hsv = color!(hsv 217 49% 90%);
    let cymk = color!(0% 73% 78% 8%);
    let rgb = color!(235, 63, 52);
    let hex = color!(#f43f5e);
    let xterm = color!(214);
    let sys = color!(green);

    println!("{hsl:#}HSL{hsl:-#}");
    println!("{hsv:#}HSV{hsv:-#}");
    println!("{cymk:#}CYMK{cymk:-#}");
    println!("{rgb:#}RGB{rgb:-#}");
    println!("{hex:#}HEX{hex:-#}");
    println!("{xterm:#}XTERM{xterm:-#}");
    println!("{sys:#}System Color{hex:-#}");

    /*
        Styling flags.
            Other styling is built from flags. These flags use binary bit mapping to save on space.
            Use the binary or operator (`|`) to add flags together. Use the binary and operator (`&`) to
            check if a flag is present; Ex: `flags & BOLD == BOLD` will be true if BOLD is present.

            If `RESET` is present, it will take priority over all other flags when the reset sequence is
            applied. For example if the flags are build with `BOLD | ITALIC` the sequence is `\x1b[1;3m`.
            When resetting the styling the reset sequence without `RESET` is `\x1b[22;23m`. Now add `RESET`
            to the flags, `flags = flags | RESET`. The sequence is still the same, however the reset
            sequence is now `\x1b[0m`.
     */
    let flags = BOLD | ITALIC | UNDERLINE | CROSSED | BLINK | REVERSED;
    println!("{flags}Flags: BOLD | ITALIC | UNDERLINE | CROSSED | BLINK | REVERSED{flags:-}");

    /*
        Hyperlink:
            Hyperlinks are limited in support, but most modern terminal emulators support them.
            The hyperlink style references a web url and surrounds the text that is to be the hyperlink.
            This means the is printed to start the hyperlink and then reset when it ends.
            Ex: `{link}This is a hyperlink{link:-}` where link = `Hyperlink("https://example.com")`
            This will make all the text, `This is a hyperlink` a clickable link in the terminal.

            The `Hyperlink` object takes anything that implements `Display` as to keep the api
            as flexible as possible. However, it also expects that the result of calling `to_string`
            on the object will result in a web url.

            The `Hyperlink` struct is the only struct in the library that doesn't have a ansi code and
            reset code. This means it cannot be chained or built with other ansi codes into one single
            sequence. Hyperlinks have an opening sequence and a closing sequence and that is it.
            It is meant to surround the text that is the hyperlink.
     */
    let link = Hyperlink("https://example.com");
    println!("{link}This is a hyperlink{link:-}");

    /*
        Style:
            Combine it all together in a Style struct. The struct is also a builder struct where
            it consumes itself and returns itself.
    */
    let style = Style {
        flags: BOLD | ITALIC | UNDERLINE | CROSSED | BLINK | REVERSED,
        fg: Some(color!(220, 100, 50)),
        bg: Some(color!(243)),
        link: Some(Hyperlink("https://example.com")),
    };
    println!("{style}All Together (Style){style:-}");

    // Example using the builder
    let style = Style::builder()
        .flags(ITALIC | UNDERLINE)
        .bold()
        .crossed()
        .fg(color!(yellow));
    println!("{style}All Together (Style): builder{style:-}");


    /*
        Capabilities:
            The library also provides terminal capabilities like whether it supports ansi sequences, and
            what colors are available. By default, the library won't use these capabilities and will
            always display the styling as you format it. This struct however can be used to create custom
            styling objects like color with fallback, or to make styling only apply if it is supported.
    */
    println!("{:?}", Capabilities::default());

    Ok(())
}
