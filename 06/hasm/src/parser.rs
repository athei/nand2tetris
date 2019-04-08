use regex::{Regex, RegexSet};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::str::Lines;

const REGEX_TEXT: [&str; 3] = [
    r"^@([0-9]+|[a-zA-Z0-9_\.\$:]+$",            // A-Instruction
    r"^(([^=]{1,3})=)?([^;]{1,3})(;(.{3}))?$",   // C-Instruction
    r"^\(([a-zA-Z_\.\$:][0-9a-zA-Z_\.\$:]*)\)$", // Label
];

thread_local! {
    static REGEXES: Vec<Regex> = REGEX_TEXT.iter().map(|rex| Regex::new(rex).unwrap()).collect();
    static REGEX_SET: RegexSet = RegexSet::new(&REGEX_TEXT).unwrap();
    static REGEX_STRIP_COMMENT: Regex = Regex::new(r"(?m)//.*$").unwrap();
    static REGEX_STRIP_WHITESPACE: Regex = Regex::new(r"[ \t]+").unwrap();
}

pub struct AsmLine {
    address: u16,
    command: Command,
}

pub enum Command {
    A(ACommand),
    C(u16),
    L(String),
}

pub enum ACommand {
    Constant(i16),
    Symbol(String),
}

pub struct Parser {
    text: String,
}

impl AsmLine {
    fn parse(line: &str) -> Result<Self, String> {
        let matches = REGEX_SET.with(|set| set.matches(line));
        unimplemented!()
    }
}

impl Parser {
    pub fn fromFile(path: &Path) -> std::io::Result<Self> {
        let mut text = String::new();
        let mut file = File::open(path)?;
        file.read_to_string(&mut text)?;
        Self::fromString(&text)
    }

    pub fn fromString(text: &str) -> std::io::Result<Self> {
        // strip away ignored text
        let text = REGEX_STRIP_COMMENT.with(|r| r.replace_all(&text, ""));
        let text = REGEX_STRIP_WHITESPACE.with(|r| r.replace_all(&text, ""));

        let parser = Self { text: text.into() };
        Ok(parser)
    }

    pub fn iter(&self) -> AsmLines {
        AsmLines {
            current_address: 0,
            current_line: self.text.lines(),
            failed: false,
        }
    }
}

pub struct AsmLines<'a> {
    current_address: u16,
    current_line: Lines<'a>,
    failed: bool,
}

impl<'a> Iterator for AsmLines<'a> {
    type Item = Result<AsmLine, String>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.failed {
            return None;
        }
        let ret = self
            .current_line
            .next()
            .filter(|line| !line.is_empty())
            .map(AsmLine::parse);
        if let Some(Ok(AsmLine { command, .. })) = &ret {
            match command {
                Command::L(_) => (),
                _ => self.current_address += 1,
            }
        } else {
            self.failed = true
        }
        ret
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter() {
        let asm = "\
        @3 //blub
        @4
        //blubbl
        ";

        let parser = Parser::fromString(asm).unwrap();

        assert_eq!(parser.text, "@3\n@4\n\n");
    }
}
