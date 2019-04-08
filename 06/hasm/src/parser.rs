use regex::{Regex, RegexSet, Captures};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::str::Lines;

const REGEX_TEXT: [&str; 3] = [
    r"^@(?P<content>([0-9]+)|[a-zA-Z0-9_\.\$:])+$",                   // A-Instruction
    r"^((?P<dest>[^=]{1,3})=)?(?P<op>[^@;]{1,3})(;(?P<jmp>.{3}))?$", // C-Instruction
    r"^\((?P<content>[a-zA-Z_\.\$:][0-9a-zA-Z_\.\$:]*)\)$",         // Label
];

thread_local! {
    static REGEXES: Vec<Regex> = REGEX_TEXT.iter().map(|rex| Regex::new(rex).unwrap()).collect();
    static REGEX_SET: RegexSet = RegexSet::new(&REGEX_TEXT).unwrap();
    static REGEX_STRIP_COMMENT: Regex = Regex::new(r"(?m)//.*$").unwrap();
    static REGEX_STRIP_WHITESPACE: Regex = Regex::new(r"[ \t]+").unwrap();
}

#[derive(Debug)]
pub struct AsmLine {
    address: u16,
    command: Command,
}

#[derive(Debug)]
pub enum Command {
    A(ACommand),
    C(u16),
    L(String),
}

#[derive(Debug)]
pub enum ACommand {
    Constant(u16),
    Symbol(String),
}

pub struct Parser {
    text: String,
}

impl AsmLine {
    fn parse(loc: u32, address: u16, line: &str) -> Result<Self, String> {
        let matches = REGEX_SET.with(|set| set.matches(line));
        if let Some(idx) = matches.into_iter().nth(0) {
            let cap = REGEXES.with(|r| r[idx].captures_iter(line).nth(0).unwrap());

            let command = match idx {
                0 => Command::A(Self::parse_a(&cap)),
                1 => Command::C(Self::parse_c(&cap)?),
                2 => Command::L(Self::parse_l(&cap)),
                _ => panic!("This should be exaustive"),
            };

            let asm =  AsmLine {
                address,
                command
            };

            return Ok(asm);
        }
        Err(format!("{}: Cannot recognize {} at address {}", loc, line, address))
    }

    fn parse_a(cap: &Captures) -> ACommand {
        let content = &cap["content"];

        if let Ok(num) = u16::from_str_radix(content, 10) {
            return ACommand::Constant(num);
        }

        ACommand::Symbol(content.into())
    }

    fn parse_c(cap: &Captures) -> Result<u16, String> {
        unimplemented!()
    }

    fn parse_l(cap: &Captures) -> String {
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
            current_loc: 0,
            current_line: self.text.lines(),
            failed: false,
        }
    }
}

pub struct AsmLines<'a> {
    current_address: u16,
    current_loc: u32,
    current_line: Lines<'a>,
    failed: bool,
}

impl<'a> Iterator for AsmLines<'a> {
    type Item = Result<AsmLine, String>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut line = "";

        if self.failed {
            return None;
        }

        while line.is_empty() {
            line = self.current_line.next()?;
            self.current_loc += 1;
        }

        let line = AsmLine::parse(self.current_loc, self.current_address, line);

        /* increment address if command is NOT a label */
        if let Ok(AsmLine { command, .. }) = &line {
            match command {
                Command::L(_) => (),
                _ => self.current_address += 1,
            }
        } else {
            self.failed = true
        }

        Some(line)
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

    #[test]
    fn parse() {
        let asm = "
        @-5
        @3 //blub
        @4
        //blubbl
        ";

        let parser = Parser::fromString(asm).unwrap();

        for line in parser.iter() {
            println!("{:?}", line);
        }
    }
}
