use std::borrow::Cow::{self, Borrowed, Owned};


use rustyline::completion::FilenameCompleter;

use rustyline::highlight::{Highlighter};
use rustyline::hint::HistoryHinter;
use rustyline::validate::MatchingBracketValidator;
use rustyline_derive::{Completer, Helper, Hinter, Validator};



#[derive(Helper, Completer, Hinter, Validator)]
pub struct LineHelper {
    #[rustyline(Completer)]
    pub completer: FilenameCompleter,
    pub highlighter: CommandHighlighter,
    #[rustyline(Validator)]
    pub validator: MatchingBracketValidator,
    #[rustyline(Hinter)]
    pub hinter: HistoryHinter,
    pub colored_prompt: String,
    // pub ctx:Config_class // TODO
}

impl Highlighter for LineHelper {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> Cow<'b, str> {
        if default {
            Borrowed(&self.colored_prompt)
        } else {
            Borrowed(prompt)
        }
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Owned("\x1b[1m".to_owned() + hint + "\x1b[m")
    }

    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        self.highlighter.highlight(line, pos)
    }

    fn highlight_char(&self, line: &str, pos: usize) -> bool {
        self.highlighter.highlight_char(line, pos)
    }
}

// To debug rustyline:
// RUST_LOG=rustyline=debug cargo run --example example 2> debug.log
// /// ////// ///// ///// //// //// //// /// /
// const BOPENS: &[u8; 3] = b"\"\'`";
// const BCLOSES: &[u8; 6] = b"}])\"\'`";
/// Highlight matching bracket when typed or cursor moved on.
// #[derive()]
pub struct CommandHighlighter {
}

impl CommandHighlighter {
    /// Constructor
    #[must_use]
    pub fn new() -> Self {
        Self {
        }
    }
}
impl Highlighter for CommandHighlighter {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        if line.len()<3{
            return  Borrowed(line);
        }
        let exe = line.trim().split_ascii_whitespace().next().unwrap_or_default();
        let executable_color = if which::which(exe).is_ok(){"\x1b[0;35m"} else {"\x1b[1;31m"};
        let mut copy = line.to_owned();
        let index = line.find(" ").unwrap_or_default();
        copy.replace_range(0..index,&format!("{}{}\x1b[0m", executable_color,exe));
        return Owned(copy);
    
        
        
        // Borrowed(line)
    }

    fn highlight_char(&self, line: &str, _pos: usize) -> bool {
        // will highlight matching brace/bracket/parenthesis if it exists

        // self.bracket.set(check_bracket(line, pos));
        // self.bracket.get().is_some()
        line.contains("\"")||line.contains("\'")||line.contains("\"")||line.contains(" ")
    }
}
