extern crate byteorder;
extern crate checked_int_cast;

use std::io;
use std::io::Write;
use byteorder::{LittleEndian, WriteBytesExt};
use checked_int_cast::CheckedIntCast;

pub const MACHINE_I386: u16 = 0x014c;
pub const MACHINE_IA64: u16 = 0x0200;
pub const MACHINE_AMD64: u16 = 0x8664;


pub struct Section {
    pub name: String,
    pub data: Vec<u8>,
    pub characteristics: u32,
    pub relocations: Vec<Relocation>,
}
pub struct Relocation {
    pub virtual_address: u32,
    pub symbol_index: u32,
    pub type_flags: u16
}
pub struct Symbol {
    pub name: String,
    pub value: u32,
    pub section_number: i16,
    pub type_flags: u16,
    pub storage_class: u8,
    pub aux_symbols: Vec<[u8; 18]>,
}

pub struct Coff {
    pub machine: u16,
    pub timestamp: u32,
    pub sections: Vec<Section>,
    pub symbols: Vec<Symbol>,
    pub optional_header: Vec<u8>,
    pub characteristics: u16,
}

impl Coff {
    pub fn write<W: Write>(&self, wtr: &mut W) -> io::Result<()> {
        let header_length = 20; assert!(self.optional_header.len() == 0);
        let section_headers_length = self.sections.len() * 40;
        let section_body_start = (header_length + section_headers_length) as u32; // TODO
        let section_body_total_len = self.sections.iter().fold(0, |sum, section| sum + section.data.len()) as u32;
        let relocation_start = section_body_start + section_body_total_len;
        let total_relocation_count = self.sections.iter().fold(0, |sum, section| sum + section.relocations.len());
        let symbol_table_start = relocation_start + (total_relocation_count as u32)*10;
        let symbol_table_count = self.symbols.iter().fold(0, |sum, symbol| sum + 1 + symbol.aux_symbols.len()) as u32;
        let symbol_table_length = symbol_table_count * 18;
        
        let mut string_table: Vec<String> = Vec::new();
        
        
        try!(wtr.write_u16::<LittleEndian>(self.machine));
        try!(wtr.write_u16::<LittleEndian>(try!(self.sections.len().as_u16_checked().ok_or_else(|| io::Error::new(io::ErrorKind::Other, "too many sections in COFF")))));
        try!(wtr.write_u32::<LittleEndian>(self.timestamp));
        try!(wtr.write_u32::<LittleEndian>(symbol_table_start));
        try!(wtr.write_u32::<LittleEndian>(symbol_table_count));
        try!(wtr.write_u16::<LittleEndian>(try!(self.optional_header.len().as_u16_checked().ok_or_else(|| io::Error::new(io::ErrorKind::Other, "optional header too large")))));
        try!(wtr.write_u16::<LittleEndian>(self.characteristics));
        
        let mut data_offset = section_body_start;
        let mut relocation_offset = relocation_start;
        let mut string_table_offset = 4u32;
        for section in self.sections.iter() {
            
            // TODO: COFF only supports ASCII, and they'd better not start with a forward slash
            let name_bytes = section.name.as_bytes();
            
            if name_bytes.len() <= 8 {
                try!(wtr.write_all(name_bytes));
                try!(wtr.write_all(&vec![0u8; 8 - name_bytes.len()][..]));
            } else {
                let string_table_str = format!("/{}", string_table_offset).into_bytes();
                try!(wtr.write_all(&string_table_str[..]));
                try!(wtr.write_all(&vec![0u8; 8 - string_table_str.len()][..]));
                
                string_table.push(section.name.clone()); // TODO: clone?!
                string_table_offset += name_bytes.len() as u32 + 1;
            }
            
            //let data_offset = 0x1234u32; // TODO
            let line_number_offset = 0;
            let line_number_count = 0;
            
            try!(wtr.write_u32::<LittleEndian>(0)); // VirtualSize - 0 for object files
            try!(wtr.write_u32::<LittleEndian>(0)); // VirtualAddress - 0 for object files
            try!(wtr.write_u32::<LittleEndian>(try!(section.data.len().as_u32_checked().ok_or_else(|| io::Error::new(io::ErrorKind::Other, "data for COFF section too long")))));
            try!(wtr.write_u32::<LittleEndian>(if section.data.len()>0 { data_offset } else { 0 }));
            try!(wtr.write_u32::<LittleEndian>(if section.relocations.len()>0 { relocation_offset } else { 0 }));
            try!(wtr.write_u32::<LittleEndian>(line_number_offset));
            try!(wtr.write_u16::<LittleEndian>(section.relocations.len() as u16));
            try!(wtr.write_u16::<LittleEndian>(line_number_count));
            try!(wtr.write_u32::<LittleEndian>(section.characteristics));
            
            data_offset += section.data.len() as u32;
            relocation_offset += section.relocations.len() as u32 * 10;
            // TODO: Round data_offset up to nearest 4?
        }
        
        for section in self.sections.iter() {
            try!(wtr.write_all(&section.data[..]));
        }
        
        for section in self.sections.iter() {
            for relocation in section.relocations.iter() {
                try!(wtr.write_u32::<LittleEndian>(relocation.virtual_address));
                try!(wtr.write_u32::<LittleEndian>(relocation.symbol_index));
                try!(wtr.write_u16::<LittleEndian>(relocation.type_flags));
            }
        }
        
        for symbol in self.symbols.iter() {
            // TODO: COFF only supports ASCII, and they'd better not start with a forward slash
            let name_bytes = symbol.name.as_bytes();
            
            if name_bytes.len() <= 8 {
                try!(wtr.write_all(name_bytes));
                try!(wtr.write_all(&vec![0u8; 8 - name_bytes.len()][..]));
            } else {
                try!(wtr.write_u32::<LittleEndian>(0u32));
                try!(wtr.write_u32::<LittleEndian>(string_table_offset));
                
                string_table.push(symbol.name.clone()); // TODO: clone?!
                string_table_offset += name_bytes.len() as u32 + 1;
            }
            
            try!(wtr.write_u32::<LittleEndian>(symbol.value));
            try!(wtr.write_i16::<LittleEndian>(symbol.section_number));
            try!(wtr.write_u16::<LittleEndian>(symbol.type_flags));
            try!(wtr.write_u8(symbol.storage_class));
            try!(wtr.write_u8(symbol.aux_symbols.len() as u8)); // TODO
            for aux_symbol in symbol.aux_symbols.iter() {
                try!(wtr.write_all(&aux_symbol[..]));
            }
        }
        
        try!(wtr.write_u32::<LittleEndian>(string_table_offset));
        for string in string_table {
            try!(wtr.write_all(string.as_bytes()));
            try!(wtr.write_all(&[0u8][..]));
        }
        
        Ok( () )
    }
}

#[cfg(test)]
mod tests {
    use super::{Coff, Section, Relocation, Symbol, MACHINE_AMD64};
    use std::fs::File;
    use std::io::Read;
    
    #[test]
    fn it_works() {
        let c = Coff{
            machine: MACHINE_AMD64,
            timestamp: 0,
            optional_header: Vec::new(),
            characteristics: 0x0004,
            sections: vec![
                Section{
                    name: ".text".to_string(),
                    characteristics: 0x60500020,
                    data: vec![
                        0x8D, 0x41, 0x05, 0xC3, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90
                    ],
                    relocations: vec![],
                },
                Section{
                    name: ".data".to_string(),
                    characteristics: 0xC0500040,
                    data: vec![ ],
                    relocations: vec![],
                },
                Section{
                    name: ".bss".to_string(),
                    characteristics: 0xC0500080,
                    data: vec![ ],
                    relocations: vec![],
                },
                Section{
                    name: ".text.unlikely".to_string(), // Gets rendered as /4, so I'm making this long
                    characteristics: 0x60500020,
                    data: vec![ ],
                    relocations: vec![],
                },
                Section{
                    name: ".xdata".to_string(),
                    characteristics: 0x40300040,
                    data: vec![ 0x01, 0x00, 0x00, 0x00 ],
                    relocations: vec![],
                },
                Section{
                    name: ".pdata".to_string(),
                    characteristics: 0x40300040,
                    data: vec![ 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00 ],
                    relocations: vec![
                        Relocation{
                            virtual_address: 0,
                            symbol_index: 4,
                            type_flags: 3
                        },
                        Relocation{
                            virtual_address: 4,
                            symbol_index: 4,
                            type_flags: 3
                        },
                        Relocation{
                            virtual_address: 8,
                            symbol_index: 10,
                            type_flags: 3
                        }
                        //OFFSET           TYPE              VALUE
                        //0000000000000000 rva32             .text
                        //0000000000000004 rva32             .text
                        //0000000000000008 rva32             .xdata
                    ],
                },
                Section{
                    name: ".rdata$zzz".to_string(),
                    characteristics: 0x40500040,
                    data: vec![ 0x47, 0x43, 0x43, 0x3A, 0x20, 0x28, 0x78, 0x38, 0x36, 0x5F, 0x36, 0x34, 0x2D, 0x70, 0x6F, 0x73, 0x69, 0x78, 0x2D, 0x73, 0x65, 0x68, 0x2D, 0x72, 0x65, 0x76, 0x30, 0x2C, 0x20, 0x42, 0x75, 0x69, 0x6C, 0x74, 0x20, 0x62, 0x79, 0x20, 0x4D, 0x69, 0x6E, 0x47, 0x57, 0x2D, 0x57, 0x36, 0x34, 0x20, 0x70, 0x72, 0x6F, 0x6A, 0x65, 0x63, 0x74, 0x29, 0x20, 0x35, 0x2E, 0x31, 0x2E, 0x30, 0x00, 0x00 ],
                    relocations: vec![],
                },
            
            ],
            symbols: vec![
                Symbol {
                    name: ".file".to_string(),
                    value: 0,
                    section_number: -2,
                    type_flags: 0,
                    storage_class: 0x67,
                    aux_symbols: vec![
                        [0x66, 0x6F, 0x6F, 0x2E, 0x63, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
                    ]
                },
                Symbol {
                    name: "foo".to_string(),
                    value: 0,
                    section_number: 1,
                    type_flags: 0x20,
                    storage_class: 0x02,
                    aux_symbols: vec![
                        [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
                    ]
                },
                Symbol {
                    name: ".text".to_string(),
                    value: 0,
                    section_number: 1,
                    type_flags: 0,
                    storage_class: 3,
                    aux_symbols: vec![
                        [0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
                    ]
                },
                Symbol {
                    name: ".data".to_string(),
                    value: 0,
                    section_number: 2,
                    type_flags: 0,
                    storage_class: 3,
                    aux_symbols: vec![
                        [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
                    ]
                },
                Symbol {
                    name: ".bss".to_string(),
                    value: 0,
                    section_number: 3,
                    type_flags: 0,
                    storage_class: 3,
                    aux_symbols: vec![
                        [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
                    ]
                },
                Symbol {
                    name: ".xdata".to_string(),
                    value: 0,
                    section_number: 5,
                    type_flags: 0,
                    storage_class: 3,
                    aux_symbols: vec![
                        [0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
                    ]
                },
                Symbol {
                    name: ".pdata".to_string(),
                    value: 0,
                    section_number: 6,
                    type_flags: 0,
                    storage_class: 3,
                    aux_symbols: vec![
                        [0x0C, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
                    ]
                },
                Symbol {
                    name: ".rdata$zzz".to_string(),
                    value: 0,
                    section_number: 7,
                    type_flags: 0,
                    storage_class: 3,
                    aux_symbols: vec![
                        [0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
                    ]
                },
            ]
            
        };
        
        let mut f = File::create("out.o").expect("file open failed");
        c.write(&mut f).unwrap();
        
        {
            let mut xs = Vec::new();
            let mut ys = Vec::new();
            File::open("out.o").unwrap().read_to_end(&mut xs).unwrap();
            File::open("..\\buildwithgcc\\foo.o").unwrap().read_to_end(&mut xs).unwrap();
            
            for (idx, (x, y)) in xs.iter().zip(ys.iter()).enumerate() {
                if *x != *y {
                    panic!("differ at {}", idx);
                }
            }
        }
    
    }
}
