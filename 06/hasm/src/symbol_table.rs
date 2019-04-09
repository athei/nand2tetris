use crate::parser::{AsmLine, Command::L, Parser};
use std::collections::HashMap;

pub struct SymbolTable {
    table: HashMap<String, u16>,
    current_ram: u16,
}

impl SymbolTable {
    pub fn new(parser: &Parser) -> Result<Self, String> {
        let mut table: HashMap<String, u16> = [
            ("SP".into(), 0x00_00),
            ("LCL".into(), 0x00_01),
            ("ARG".into(), 0x00_02),
            ("THIS".into(), 0x00_03),
            ("THAT".into(), 0x00_04),
            ("R0".into(), 0x00_00),
            ("R1".into(), 0x00_01),
            ("R2".into(), 0x00_02),
            ("R3".into(), 0x00_03),
            ("R4".into(), 0x00_04),
            ("R5".into(), 0x00_05),
            ("R6".into(), 0x00_06),
            ("R7".into(), 0x00_07),
            ("R8".into(), 0x00_08),
            ("R9".into(), 0x00_09),
            ("R10".into(), 0x00_0a),
            ("R11".into(), 0x00_0b),
            ("R12".into(), 0x00_0c),
            ("R13".into(), 0x00_0d),
            ("R14".into(), 0x00_0e),
            ("R15".into(), 0x00_0f),
            ("SCREEN".into(), 0x40_00),
            ("KBD".into(), 0x60_00),
        ]
        .iter()
        .cloned()
        .collect();

        // first pass for rom addresses
        for line in parser.iter() {
            if let Ok(AsmLine {
                address,
                command: L(sym),
                ..
            }) = line
            {
                let old = table.insert(sym.clone(), address);
                if let Some(old) = old {
                    return Err(format!("{} was already allocated to address {}", sym, old));
                }
            }
        }

        let sym = Self {
            table,
            current_ram: 0x00_10,
        };
        Ok(sym)
    }

    pub fn get_address(&mut self, sym: &str) -> u16 {
        if let Some(address) = self.table.get(sym) {
            return *address;
        }

        self.table.insert(sym.to_string(), self.current_ram);
        let old_ram = self.current_ram;
        self.current_ram += 1;
        old_ram
    }
}
