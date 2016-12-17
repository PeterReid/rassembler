extern crate byteorder;

use std::io;
use std::io::Write;

use byteorder::{LittleEndian, WriteBytesExt};

#[derive(Copy, Clone)]
pub enum WordSize {
    Bits32, Bits64
}
impl WordSize {
    fn as_u8(self) -> u8 {
        match self {
            WordSize::Bits32 => 1,
            WordSize::Bits64 => 2,
        }
    }
}
#[derive(Copy, Clone)]
pub enum Endianness {
    LittleEndian,
    BigEndian,
}
impl Endianness {
    fn as_u8(self) -> u8 {
        match self {
            Endianness::LittleEndian => 1,
            Endianness::BigEndian => 2,
        }
    }
}
#[derive(Copy, Clone)]
pub enum Architecture {
    X86,
    Arm,
    X8664,
    Avr,
}
impl Architecture {
    fn as_u8(self) -> u8 {
        match self {
            Architecture::X86 => 0x03,
            Architecture::Arm => 0x27,
            Architecture::X8664 => 0x3E,
            Architecture::Avr => 0x53,
        }
    }
}

pub struct Function<'a> {
    pub offset: usize,
    pub name: &'a str,
}

pub struct Elf<'a> {
    pub word_size: WordSize,
    pub endianness: Endianness,
    pub architecture: Architecture,
    pub file_name: &'a str,
    pub functions: Vec<Function<'a>>,
    pub text_content: &'a [u8],
}


struct SectionHeader<'a> {
    name: &'a str,
    section_type: u32,
    flags: u32,
    address: u64,
    content: &'a [u8],
    link: u32,
    info: u32,
    align: u64,
    entsize: u64,
}

struct StringTable {
    inner: Vec<u8>,
}
impl StringTable {
    fn new() -> StringTable {
        StringTable{
            inner: vec![0]
        }
    }

    fn append(&mut self, s: &str) -> u64 {
        if s.len() == 0 {
            return 0;
        }

        let ret = self.inner.len() as u64;
        for b in s.as_bytes().iter() {
            self.inner.push(*b);
        }
        self.inner.push(0);
        ret
    }
}

struct Symbol<'a> {
    name: &'a str,
    offset: u64,
    size: u64,
    info: u8,
    other: u8,
    shndx: u16,
}

impl<'a> Elf<'a> {
    pub fn write<W: Write>(&self, w: &mut W) -> Result<(), io::Error> {
        let mut symbols = vec![
            Symbol { name: "", offset: 0, size: 0, info: 0, other: 0, shndx: 0 }, // blank entry
            Symbol { name: self.file_name, offset: 0, size: 0, info: 4, other: 0, shndx: 0xfff1 }, // file
            Symbol { name: "", offset: 0, size: 0, info: 3, other: 0, shndx: 1 }, // section
        ];
        for function in self.functions.iter() {
            symbols.push(Symbol { name: function.name, offset: function.offset as u64, size: 0, info: 0x10, other: 0, shndx: 1 });
        };

        let index_of_section_name_table = 2u16; // of .shstrtab

        let mut symbol_string_table = StringTable::new();
        let symbol_table_contents = {
            let mut symbol_table_contents = Vec::new();
            for symbol in symbols.iter() {
                let name_offset = symbol_string_table.append(&symbol.name);
                match self.word_size {
                    WordSize::Bits32 => {
                        try!(symbol_table_contents.write_u32::<LittleEndian>(name_offset as u32));
                        try!(symbol_table_contents.write_u32::<LittleEndian>(symbol.offset as u32));
                        try!(symbol_table_contents.write_u32::<LittleEndian>(symbol.size as u32));
                        try!(symbol_table_contents.write_u8(symbol.info));
                        try!(symbol_table_contents.write_u8(symbol.other));
                        try!(symbol_table_contents.write_u16::<LittleEndian>(symbol.shndx));
                    }
                    WordSize::Bits64 => {
                        try!(symbol_table_contents.write_u32::<LittleEndian>(name_offset as u32));
                        try!(symbol_table_contents.write_u8(symbol.info));
                        try!(symbol_table_contents.write_u8(symbol.other));
                        try!(symbol_table_contents.write_u16::<LittleEndian>(symbol.shndx));
                        try!(symbol_table_contents.write_u64::<LittleEndian>(symbol.offset));
                        try!(symbol_table_contents.write_u64::<LittleEndian>(symbol.size));
                    }
                }
            }
            symbol_table_contents
        };
 
        let mut section_strings = StringTable::new();
        let mut section_headers = [
            SectionHeader{
                name: "",
                section_type: 0,
                flags: 0,
                address: 0,
                content: &[][..],
                link: 0,
                info: 0,
                align: 0,
                entsize: 0,
            },
            SectionHeader{
                name: ".text",
                section_type: 1,
                flags: 6, // ???
                address: 0,
                content: self.text_content,
                link: 0,
                info: 0,
                align: 16,
                entsize: 0,
            },
            SectionHeader{
                name: ".shstrtab",
                section_type: 3,
                flags: 0,
                address: 0,
                content: &[][..], // we'll fill this in shortly
                link: 0,
                info: 0,
                align: 1,
                entsize: 0,
            },
            SectionHeader{
                name: ".symtab",
                section_type: 2,
                flags: 0,
                address: 0,
                content: &symbol_table_contents[..],
                link: 4,
                info: 3,
                align: 4,
                entsize: 0x18,
            },
            SectionHeader{
                name: ".strtab",
                section_type: 3,
                flags: 0,
                address: 0,
                content: &symbol_string_table.inner[..],
                link: 0,
                info: 0,
                align: 1,
                entsize: 0,
            },
        ];

        let mut section_name_offsets = Vec::new();
        for section_header in section_headers.iter() {
            section_name_offsets.push(section_strings.append(section_header.name));
        }

        section_headers[index_of_section_name_table as usize].content = &section_strings.inner[..];

        try!(w.write_all(b"\x7fELF"));
        try!(w.write_u8(self.word_size.as_u8()));
        try!(w.write_u8(self.endianness.as_u8()));
        try!(w.write_u8(1));
        try!(w.write_u8(0)); // operating system... we will just leave this 0
        try!(w.write_all(&[0u8; 8][..]));
        try!(w.write_u16::<LittleEndian>(1u16)); // relocatable 
        try!(w.write_u16::<LittleEndian>(self.architecture.as_u8() as u16));
        try!(w.write_u32::<LittleEndian>(1u32)); // original ELF version
        try!(self.write_word(w, 0)); // entry point -- since this is not an executable, just 0
        try!(self.write_word(w, 0)); // program header table offset
        try!(self.write_word(w, match self.word_size { // section header start
            WordSize::Bits32 => 0x34,
            WordSize::Bits64 => 0x40,
        }));
        try!(w.write_u32::<LittleEndian>(0)); // flags
        try!(w.write_u16::<LittleEndian>(match self.word_size {
            WordSize::Bits32 => 0x34,
            WordSize::Bits64 => 0x40,
        }));
        try!(w.write_u16::<LittleEndian>(0)); // program header entry size
        try!(w.write_u16::<LittleEndian>(0)); // program header entry count
        try!(w.write_u16::<LittleEndian>(match self.word_size {
            WordSize::Bits32 => 0x28,
            WordSize::Bits64 => 0x40,
        })); // section header entry size
        try!(w.write_u16::<LittleEndian>(section_headers.len() as u16)); // section header entry count
        try!(w.write_u16::<LittleEndian>(index_of_section_name_table)); // not sure yet...


        let mut offset = 0x180;
        for (section_header, name_offset) in section_headers.iter().zip(section_name_offsets.iter()) {
            if section_header.content.len() != 0 {
                offset = (offset + 15) & !15;
            }
            try!(w.write_u32::<LittleEndian>(*name_offset as u32));
            try!(w.write_u32::<LittleEndian>(section_header.section_type));
            try!(self.write_word(w, section_header.flags as u64));
            try!(self.write_word(w, section_header.address));
            try!(self.write_word(w, if section_header.content.len() > 0 { offset } else { 0 }));
            try!(self.write_word(w, section_header.content.len() as u64));
            try!(w.write_u32::<LittleEndian>(section_header.link));
            try!(w.write_u32::<LittleEndian>(section_header.info));
            try!(self.write_word(w, section_header.align));
            try!(self.write_word(w, section_header.entsize));
            offset += section_header.content.len() as u64;
        }

        for section in section_headers.iter() {
            try!(w.write_all(section.content));
            try!(w.write_all(&[0u8; 16][.. (16 - section.content.len()%16) % 16]));
        }

        Ok( () )
    }

    fn write_word<W: Write>(&self, w: &mut W, value: u64) -> io::Result<()> {
        match self.word_size {
            WordSize::Bits64 => w.write_u64::<LittleEndian>(value),
            WordSize::Bits32 => w.write_u32::<LittleEndian>(value as u32),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Elf, Architecture, WordSize, Endianness, Function};
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn it_works() {
        let mut xs = Vec::new();
        let e = Elf {
            architecture: Architecture::X8664,
            word_size: WordSize::Bits64,
            endianness: Endianness::LittleEndian,
            file_name: "fooasm.asm",
            functions: vec![
                Function{
                    name: "foo",
                    offset: 0,
                },
                Function{
                    name: "bar",
                    offset: 6,
                },
            ],
            text_content: &[0xb8, 0x04, 0x00, 0x00, 0x00, 0xc3, 0xb8, 0x09, 0x00, 0x00, 0x00, 0xc3][..],
        };
        e.write(&mut xs).unwrap();
        File::create("out.elf").expect("open failed").write_all(&xs[..]).expect("write failed");
    }
}
