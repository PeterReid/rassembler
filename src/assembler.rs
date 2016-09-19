use compiler::{StmtBuffer, Stmt, Opdata, compile_op};
use parser::{Ident, Arg, Size, ImmediateValue};
use std::ops::{Deref, DerefMut};
use object_file::{ObjectFile, ExportedFunction};

use std::fs::File;
use std::io::ErrorKind;
use std::io;
use std::env;
use std::convert::Into;
use byteorder::{LittleEndian, ByteOrder};



pub struct Assembler {
    inner: FlaggedAssembler,
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
        }
    }

    pub fn global(&mut self, name: &str) {
        self.inner.buffer.push(Stmt::GlobalLabel(name.to_string()));
    }
    
    pub fn dump(&self) -> ObjectFile {
        let mut result = ObjectFile{
            code: Vec::new(),
            functions: Vec::new(),
        };
        for stmt in &self.inner.buffer {
            match *stmt {
                Stmt::Const(x) => { result.code.push(x); }
                Stmt::Var(ImmediateValue::I64(x), Size::BYTE) => {
                    result.code.push(x as u8);
                }
                Stmt::Var(ImmediateValue::I64(x), Size::DWORD) => {
                    let mut xs = [0u8; 4];
                    LittleEndian::write_i32(&mut xs[..], x as i32);
                    result.code.extend(xs.iter());
                }
                Stmt::GlobalLabel(ref ident) => {
                    result.functions.push(ExportedFunction{
                        offset: result.code.len() as u32,
                        name: ident.clone(),
                    });
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
        let out_dir = env::var("OUT_DIR").unwrap();
        let lib_name = "foo";
        
        let mut out = File::create(format!("{}/lib{}.a", out_dir, lib_name)).unwrap();
        self.dump().write(&mut out);
        println!("cargo:rustc-flags=-L native={} -l static={}", out_dir, lib_name);
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
