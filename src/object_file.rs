use coff_writer::{Coff, Section, Symbol, MACHINE_AMD64};
use elf_writer::{self, Elf};
use std::io::Write;
use std::io::ErrorKind;
use std::io;


pub struct ExportedFunction {
    pub offset: u32,
    pub name: String,
}

pub struct ObjectFile {
    pub functions: Vec<ExportedFunction>,
    pub code: Vec<u8>,

}

fn write_archive_header<W: Write>(w: &mut W, singleton_file_contents: &[u8]) -> io::Result<()> {
    try!(w.write_all(b"!<arch>\n"));
    try!(w.write_all(b"rasm.o/         "));
    try!(w.write_all(b"1468364067  ")); // Date-modified time of the file within, technically. But I'm just leaving this fix.
    try!(w.write_all(b"0     ")); // Owner ID
    try!(w.write_all(b"0     ")); // Group ID
    try!(w.write_all(b"100666  ")); // File mode
    
    let mut length_str = format!("{}", singleton_file_contents.len()).into_bytes();
    while length_str.len() < 10 {
        length_str.push(b' ');
    }
    if length_str.len() > 10 {
        return Err(io::Error::new(ErrorKind::Other, "write_archive_header attempted to archive a too-large file"));
    }
    try!(w.write_all(&length_str[..]));
    try!(w.write_all(b"`\n")); // "File magic"
    
    Ok( () )
}

fn write_archive_footer<W: Write>(w: &mut W, singleton_file_contents: &[u8]) -> io::Result<()> {
    if singleton_file_contents.len() % 2 == 1 {
        try!(w.write_all(b"\n"));
    }
    Ok( () )
}


impl ObjectFile {
    pub fn write<W: Write>(&self, out: &mut W) {
        let template_bytes =
            if false {
                self.make_elf_file()
            } else {
                self.make_object_file()
            };
 
        write_archive_header(out, &template_bytes[..]).unwrap();
        out.write_all(&template_bytes[..]).unwrap();
        write_archive_footer(out, &template_bytes[..]).unwrap();
    }
    
    fn make_elf_file(&self) -> Vec<u8> {
        let e = Elf{
            architecture: elf_writer::Architecture::X8664,
            word_size: elf_writer::WordSize::Bits64,
            endianness: elf_writer::Endianness::LittleEndian,
            file_name: "fooasm.asm",
            functions: self.functions.iter().map(|f| elf_writer::Function{
                    name: &f.name,
                    offset: f.offset as usize,
                }).collect(),
            text_content: &self.code[..], 
        };
        let mut result = Vec::new();
        e.write(&mut result).unwrap();
        result
    }
    
    fn make_object_file(&self) -> Vec<u8> {
        let mut c = Coff{
            machine: MACHINE_AMD64,
            timestamp: 0,
            optional_header: Vec::new(),
            characteristics: 0x0004,
            sections: vec![
                Section{
                    name: ".text".to_string(),
                    characteristics: 0x60500020,
                    data: self.code.to_vec(),
                    relocations: vec![],
                },
                /*Section{
                    name: ".data".to_string(),
                    characteristics: 0xC0500040,
                    data: vec![ ],
                    relocations: vec![],
                },*/
            ],
            symbols: vec![
                /*Symbol {
                    name: ".file".to_string(),
                    value: 0,
                    section_number: -2,
                    type_flags: 0,
                    storage_class: 0x67,
                    aux_symbols: vec![
                        [0x66, 0x6F, 0x6F, 0x2E, 0x63, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
                    ]
                },*/
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
                /*Symbol {
                    name: ".data".to_string(),
                    value: 0,
                    section_number: 2,
                    type_flags: 0,
                    storage_class: 3,
                    aux_symbols: vec![
                        [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
                    ]
                },*/
            ]
            
        };
        
        for function in self.functions.iter() {
            c.symbols.push(
                Symbol {
                    name: function.name.clone(),
                    value: function.offset,
                    section_number: 1,
                    type_flags: 0x20,
                    storage_class: 0x02,
                    aux_symbols: vec![
                        [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
                    ]
                }
            )
        }
        
        
        let mut result = Vec::new();
        c.write(&mut result).unwrap();
        result
    }
}