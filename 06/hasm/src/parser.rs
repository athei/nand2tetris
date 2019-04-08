use regex::{Captures, Regex, RegexSet};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::str::Lines;

const REGEX_TEXT: [&str; 3] = [
    r"^@(?P<content>([0-9]+)|[a-zA-Z0-9_\.\$:])+$", // A-Instruction
    r"^((?P<dest>[^=]{1,3})=)?(?P<op>[^@;]{1,3})(;(?P<jmp>.{3}))?$", // C-Instruction
    r"^\((?P<content>[a-zA-Z_\.\$:][0-9a-zA-Z_\.\$:]*)\)$", // Label
];

thread_local! {
    static REGEXES: Vec<Regex> = REGEX_TEXT.iter().map(|rex| Regex::new(rex).unwrap()).collect();
    static REGEX_SET: RegexSet = RegexSet::new(&REGEX_TEXT).unwrap();
    static REGEX_STRIP_COMMENT: Regex = Regex::new(r"(?m)//.*$").unwrap();
    static REGEX_STRIP_WHITESPACE: Regex = Regex::new(r"[ \t]+").unwrap();
}

#[derive(Debug)]
pub struct AsmLine {
    pub address: u16,
    pub command: Command,
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

            return Ok(Self { address, command });
        }
        Err(format!("{}: Cannot recognize {}", loc, line))
    }

    fn parse_a(cap: &Captures) -> ACommand {
        let content = &cap["content"];

        if let Ok(num) = u16::from_str_radix(content, 10) {
            return ACommand::Constant(num);
        }

        ACommand::Symbol(content.into())
    }

    fn parse_c(cap: &Captures) -> Result<u16, String> {
        let jmp = cap.name("jmp");
        let dest = cap.name("dest");
        let op = &cap["op"];
        let mut result: u16 = 0b1110_0000_0000_0000;

        if dest.is_none() && jmp.is_none() {
            return Err("Destination or jump must be set".into());
        }

        if let Some(jmp) = jmp {
            result |= match jmp.as_str() {
                "JGT" => 0b001,
                "JEQ" => 0b010,
                "JGE" => 0b011,
                "JLT" => 0b100,
                "JNE" => 0b101,
                "JLE" => 0b110,
                "JMP" => 0b111,
                x => return Err(format!("Invalid jmp: {}", x)),
            };
        }

        if let Some(dest) = dest {
            result |= match dest.as_str() {
                "M" => 0b001,
                "D" => 0b010,
                "MD" => 0b011,
                "A" => 0b100,
                "AM" => 0b101,
                "AD" => 0b110,
                "AMD" => 0b111,
                x => return Err(format!("Invalid dest: {}", x)),
            } << 3;
        }

        result |= match op {
            "0" => 0b0_101_010,
            "1" => 0b0_111_111,
            "-1" => 0b0_111_010,
            "D" => 0b0_001_100,
            "A" => 0b0_110_000,
            "M" => 0b1_110_000,
            "!D" => 0b0_001_101,
            "!A" => 0b0_110_001,
            "!M" => 0b1_110_001,
            "-D" => 0b0_001_111,
            "-A" => 0b0_110_011,
            "-M" => 0b1_110_011,
            "D+1" => 0b0_011_111,
            "A+1" => 0b0_110_111,
            "M+1" => 0b1_110_111,
            "D-1" => 0b0_001_110,
            "A-1" => 0b0_110_010,
            "D+A" => 0b0_000_010,
            "D-A" => 0b0_010_011,
            "A-D" => 0b0_000_111,
            "D&A" => 0b0_000_000,
            "D|A" => 0b0_010_101,
            "M-1" => 0b1_110_010,
            "D+M" => 0b1_000_010,
            "D-M" => 0b1_010_011,
            "M-D" => 0b1_000_111,
            "D&M" => 0b1_000_000,
            "D|M" => 0b1_010_101,
            x => return Err(format!("Invalid op: {}", x)),
        } << 6;

        Ok(result)
    }

    fn parse_l(cap: &Captures) -> String {
        cap["content"].to_string()
    }
}

impl Parser {
    pub fn from_file(path: &Path) -> std::io::Result<Self> {
        let mut text = String::new();
        let mut file = File::open(path)?;
        file.read_to_string(&mut text)?;
        Self::from_string(&text)
    }

    pub fn from_string(text: &str) -> std::io::Result<Self> {
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

        let parser = Parser::from_string(asm).unwrap();

        assert_eq!(parser.text, "@3\n@4\n\n");
    }

    #[test]
    fn parse() {
        let asm = "
        @5
        @3 //blub
        @4
        //blubbl
        (BLUB)
        (lo)
        A;JEQ
        ";

        let parser = Parser::from_string(asm).unwrap();

        for line in parser.iter() {
            println!("{:x?}", line);
        }
    }
}
