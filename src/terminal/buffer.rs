/*
    Need:
        - Scrolling to terminal size: horizontal and vertical
        - Line wrapping
        - Cursor
            - positioning
            - visibility
            - shape
        - Change chunks of text
        - Each character is a styled hunk
        - Write a styled block will create a style reference where each character references it.
        - The entire block is rendered in optimum way

        A way of taking the chunk of buffer, and finding its corresponding style.

        Lines 10-45 columns 10-120. Should then be able to take every character in that section
        and iterate them constructing the styling of each character and chunk without having to
        iterate all the styles.

        HashMap<Style, usize> == A way of knowing if the style is still being referenced and if it
        still needs to be in the map. How do I then get that style from the character if the style
        itself is a unique hash key?

        HashSet with an index?
*/

use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::ops::{Bound, Range, RangeBounds, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};
use crate::style::{AnsiSequence, color, Style};

struct Character {
    style: Option<u64>,
    character: char,
}

struct MappedStyle {
    style: Style,
    refs: usize
}

impl MappedStyle {
    fn increment(&mut self) {
        self.refs += 1;
    }

    /// Decrement the reference count returning true if the reference count is 0
    fn decrement(&mut self) -> bool {
        if self.refs > 0 {
            self.refs -= 1;
        }
        self.refs == 0
    }
}

struct TerminalBuffer {
    buffer: Vec<Vec<Character>>,
    styles: HashMap<u64, MappedStyle>
}

trait ReplaceRange {
    /// Inclusive lower bound
    fn start(&self) -> usize;
    /// Exclusive upper bound
    fn end(&self) -> usize;
    fn end_bounded(&self, max: usize) -> usize {
        self.end().min(max)
    }
}

impl ReplaceRange for usize {
    fn start(&self) -> usize {
        *self
    }
    fn end(&self) -> usize {
        self + 1
    }
}
impl ReplaceRange for RangeFull {
    fn start(&self) -> usize {
        0
    }
    fn end(&self) -> usize {
        usize::MAX
    }
}
impl ReplaceRange for Range<usize> {
    fn start(&self) -> usize {
        self.start
    }
    fn end(&self) -> usize {
        self.end
    }
}
impl ReplaceRange for RangeTo<usize> {
    fn start(&self) -> usize {
        0
    }
    fn end(&self) -> usize {
        self.end
    }
}
impl ReplaceRange for RangeFrom<usize> {
    fn start(&self) -> usize {
        self.start
    }
    fn end(&self) -> usize {
        usize::MAX
    }
}
impl ReplaceRange for RangeToInclusive<usize> {
    fn start(&self) -> usize {
        0
    }
    fn end(&self) -> usize {
        self.end + 1
    }
}
impl ReplaceRange for RangeInclusive<usize> {
    fn start(&self) -> usize {
        if let Bound::Included(val) = self.start_bound() { *val }
        else { 0 }
    }
    fn end(&self) -> usize {
        if let Bound::Included(val) = self.end_bound() { *val }
        else { 0 }
    }
}



impl TerminalBuffer {
    fn new() -> Self {
        TerminalBuffer {
            buffer: vec![Vec::new()],
            styles: HashMap::new()
        }
    }

    fn push<D: Display>(&mut self, chunk: D) {
        let mut last = self.buffer.last_mut().unwrap();
        for c in chunk.to_string().chars() {
            if c == '\n' {
                self.buffer.push(Vec::new());
                last = self.buffer.last_mut().unwrap();
            } else {
                last.push(Character { style: None, character: c });
            }
        }
    }

    fn push_styled<D: Display>(&mut self, style: Style, chunk: D) {
        let key = style.hash_key();
        if self.styles.contains_key(&key) {
            self.styles.get_mut(&key).unwrap().increment();
        } else {
            self.styles.insert(key, MappedStyle { style, refs: 1 });
        }

        let mut last = self.buffer.last_mut().unwrap();
        for c in chunk.to_string().chars() {
            if c == '\n' {
                self.buffer.push(Vec::new());
                last = self.buffer.last_mut().unwrap();
            } else {
                last.push(Character { style: Some(key), character: c });
            }
        }
    }

    /// Starting at the given line and column replace text until end of given chunk
    fn replace<D: Display, R1: ReplaceRange, R2: ReplaceRange>(&mut self, mut lines: R1, mut columns: R2, chunk: D) {
        if lines.start() >= self.buffer.len() || lines.end() >= self.buffer.len() {
            panic!("Line range is out of bounds: {}..{}", lines.start(), lines.end());
        }
        if lines.start() > lines.end() {
            panic!("Invalid line range: {}..{}", lines.start(), lines.end());
        }
        if columns.start() >= self.buffer[lines.start()].len() {
            panic!("Column range is out of bounds: {}..{}", columns.start(), columns.end());
        }
        if columns.start() > columns.end() {
            panic!("Invalid column range: {}..{}", columns.start(), columns.end());
        }

        // TODO: Join last line with end line at end column
        /*
            From startline & startcolumn to endline & endcolumn. Delete content and inject content
            in the gap that it leaves working to merge lines where possible.

            1. Convert replacement text to pseudo buffer
            2. Merge first chunk last line with new first line
            3. Merge new last line with last chunk first line
        */

        println!("{}..{} | {}..{}", lines.start(), lines.end(), columns.start(), columns.end());
    }
}

impl Display for TerminalBuffer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut buffer = Vec::new();

        let mut curr_style = Style::default();
        for line in self.buffer.iter() {
            let mut line_buffer = String::new();
            for character in line.iter() {
                let style = match character.style {
                    Some(key) => self.styles.get(&key).unwrap().style.clone(),
                    None => Style::default()
                };
                if curr_style != style {
                    line_buffer.push_str(curr_style.reset_sequence().as_str());
                    line_buffer.push_str(style.sequence().as_str());
                    curr_style = style;
                }
                line_buffer.push(character.character);
            }
            buffer.push(line_buffer);
        }
        if buffer.len() > 0 {
            buffer.last_mut().unwrap().push_str(curr_style.reset_sequence().as_str());
        }
        write!(f, "{}", buffer.join("\n"))
    }
}

pub fn test() {
    let mut buffer = TerminalBuffer::new();
    buffer.push_styled(Style::builder().fg(color!(red)).bold(), "First Buffer\n");
    buffer.push("    ");
    buffer.push_styled(Style::builder().bold(), "of styled text");
    {
        let line = buffer.buffer.get(0).unwrap();
        buffer.replace(0, 0..5, "Second")
    }
    println!("{}", buffer);
}
