use compiler::{StmtBuffer, Stmt, Opdata, compile_op};
use parser::{Ident, Arg, Size, ImmediateValue};
use std::ops::{Deref, DerefMut};

use std::fs::File;
use std::io::Write;
use std::io::ErrorKind;
use std::io;
use std::env;
use coff_writer::{Coff, Section, Symbol, MACHINE_AMD64};
use elf_writer::{self, Elf};
use std::convert::Into;
use byteorder::{LittleEndian, ByteOrder};

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

struct ExportedFunction {
    offset: u32,
    name: String,
}

pub struct Assembler {
    inner: FlaggedAssembler,
    functions: Vec<ExportedFunction>,
    code: Vec<u8>,
}

pub struct FlaggedAssembler {
    buffer: StmtBuffer,
    prefixes: Vec<Ident>,
}

impl FlaggedAssembler {
    fn encode(&mut self, name: &str, data: &'static [Opdata], args: Vec<Arg>) {
        compile_op(&mut self.buffer, name.to_string(), self.prefixes.clone(), args, data).expect("compile_op failed")
    }
}

include!(concat!(env!("OUT_DIR"), "/ops.rs"));

impl Assembler {
    pub fn new() -> Assembler {
        Assembler{
            inner: FlaggedAssembler{
                buffer: Vec::new(),
                prefixes: Vec::new(),
            },
            functions: vec![
                // template_bytes
                ExportedFunction{ offset: 0, name: "tempmain".to_string() }
            ],
            code: Vec::new(),
        }
    }
    
    pub fn dump(&self) -> Vec<u8> {
        let mut result = Vec::new();
        for stmt in &self.inner.buffer {
            match *stmt {
                Stmt::Const(x) => { result.push(x); }
                Stmt::Var(ImmediateValue::I64(x), Size::BYTE) => {
                    result.push(x as u8);
                }
                Stmt::Var(ImmediateValue::I64(x), Size::DWORD) => {
                    let mut xs = [0u8; 4];
                    LittleEndian::write_i32(&mut xs[..], x as i32);
                    result.extend(xs.iter());
                }
                _ => { panic!("Unimplemented statement: {:?}", stmt); }
            }
        }
        result
    }
    
    pub fn with_prefixes(&mut self, prefixes: Vec<Ident>) -> &mut FlaggedAssembler {
        self.inner.prefixes = prefixes;
        &mut self.inner
    }
    
    pub fn output(&mut self) {
        self.code = self.dump();
        let template_bytes =
            if false {
                self.make_elf_file()
            } else {
                self.make_object_file()
            };

        let out_dir = env::var("OUT_DIR").unwrap();
        let lib_name = "foo";
        
        let mut out = File::create(format!("{}/lib{}.a", out_dir, lib_name)).unwrap();
        write_archive_header(&mut out, &template_bytes[..]).unwrap();
        out.write_all(&template_bytes[..]).unwrap();
        write_archive_footer(&mut out, &template_bytes[..]).unwrap();
        println!("cargo:rustc-flags=-L native={} -l static={}", out_dir, lib_name);
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

#[test]
fn thing() {
    use parser::{Register, Size, RegId, RegKind, ImmediateValue};
    let mut x = Assembler::new();
    x.cpuid();
    x.add(Arg::Direct(Register{size: Size::BYTE, kind: RegKind::Static(RegId::RBX)}), Arg::Immediate(ImmediateValue::I64(8), None)); 
    println!("{:?}", x.dump());
    panic!("see");
}

impl Deref for Assembler {
    type Target = FlaggedAssembler;

    fn deref(&self) -> &FlaggedAssembler {
        &self.inner
    }
}

impl DerefMut for Assembler {
    fn deref_mut(&mut self) -> &mut FlaggedAssembler {
        &mut self.inner
    }
}



pub trait CollectableToArgs {
    fn collect_into(self, args: &mut Vec<Arg>);
}

impl<T: Into<Arg>> CollectableToArgs for T {
    fn collect_into(self, args: &mut Vec<Arg>) {
        args.push(self.into());
    }
}

impl <X: CollectableToArgs> CollectableToArgs for Option<X> {
    fn collect_into(self, args: &mut Vec<Arg>) {
        if let Some(x) = self {
            x.collect_into(args);
        }
    }
}

impl<X: CollectableToArgs, Y: CollectableToArgs> CollectableToArgs for (X, Y) {
    fn collect_into(self, args: &mut Vec<Arg>) {
        self.0.collect_into(args);
        self.1.collect_into(args);
    }
}

impl<X: CollectableToArgs, Y: CollectableToArgs, Z: CollectableToArgs> CollectableToArgs for (X, Y, Z) {
    fn collect_into(self, args: &mut Vec<Arg>) {
        self.0.collect_into(args);
        self.1.collect_into(args);
        self.2.collect_into(args);
    }
}
impl<W: CollectableToArgs, X: CollectableToArgs, Y: CollectableToArgs, Z: CollectableToArgs> CollectableToArgs for (W, X, Y, Z) {
    fn collect_into(self, args: &mut Vec<Arg>) {
        self.0.collect_into(args);
        self.1.collect_into(args);
        self.2.collect_into(args);
        self.3.collect_into(args);
    }
}
impl<A: CollectableToArgs, B: CollectableToArgs, C: CollectableToArgs, D: CollectableToArgs, E: CollectableToArgs> CollectableToArgs for (A, B, C, D, E) {
    fn collect_into(self, args: &mut Vec<Arg>) {
        self.0.collect_into(args);
        self.1.collect_into(args);
        self.2.collect_into(args);
        self.3.collect_into(args);
        self.4.collect_into(args);
    }
}

pub fn collect_args<T: CollectableToArgs>(x: T) -> Vec<Arg> {
    let mut args = Vec::new();
    x.collect_into(&mut args);
    args
}
