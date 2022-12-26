use std::borrow::Cow::{self, Borrowed, Owned};
use std::collections::HashMap;


use rustyline::completion::FilenameCompleter;

use rustyline::highlight::{Highlighter};
use rustyline::hint::HistoryHinter;
use rustyline::validate::MatchingBracketValidator;
use rustyline_derive::{Completer, Helper, Hinter, Validator};

use crate::builtins::{Builtin, self};



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
    builtins_map:HashMap<String, Builtin>,
    invalid_color:String,
    exe_color:String,
    builtins_color:String,
}

impl CommandHighlighter {
    /// Constructor
    #[must_use]
    pub fn new() -> Self {
        Self {
            builtins_map : builtins::build_map(),
            invalid_color: "\x1b[1;31m".to_string(), // bold and red
            exe_color:"\x1b[0;35m".to_string(), // purple
            builtins_color:"\x1b[38;2;255;215;0m".to_string() // RGB(255, 215, 0)
        }
    }
}
impl Highlighter for CommandHighlighter {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        if line.len()<3{
            return  Borrowed(line);
        }
        let exe = line.trim().split_ascii_whitespace().next().unwrap_or_default();
        let executable_color;
        if self.builtins_map.contains_key(exe){
            executable_color = self.builtins_color.clone()
        }
        else {
            executable_color = if which::which(exe).is_ok(){self.exe_color.clone()} else {self.invalid_color.clone()};
        }
        let mut copy = line.to_owned();
        let index = line.find(" ").unwrap_or_default();
        copy.replace_range(0..index,&format!("{}{}\x1b[0m", executable_color,exe));
        return Owned(copy);
    
        
        
        // Borrowed(line)
    }

    fn highlight_char(&self, line: &str, _pos: usize) -> bool {
        // will highlight matching exe if it exists

        // self.bracket.set(check_bracket(line, pos));
        // self.bracket.get().is_some()
        line.contains("\"")||line.contains("\'")||line.contains("\"")||line.contains(" ")
    }
}
