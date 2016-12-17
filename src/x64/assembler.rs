use x64::compiler::{StmtBuffer, Stmt, Opdata, compile_op};
use x64::parser::{Ident, Arg, Size, JumpTarget, ImmediateValue};
use std::ops::{Deref, DerefMut};
use object_file::{ObjectFile, ExportedFunction};
use std::collections::{HashMap, HashSet};

use std::fs::File;
use std::env;
use std::convert::Into;
use byteorder::{LittleEndian, ByteOrder};



pub struct Assembler {
    inner: FlaggedAssembler,
}

pub struct FlaggedAssembler {
    buffer: StmtBuffer,
    prefixes: Vec<Ident>,
    jump_target_counter: JumpTarget,
    allocated_jump_targets: HashSet<JumpTarget>,
}

impl FlaggedAssembler {
    fn encode(&mut self, name: &str, data: &'static [Opdata], args: Vec<Arg>) {
        compile_op(&mut self.buffer, name.to_string(), self.prefixes.clone(), args, data).expect("compile_op failed")
    }
}

include!(concat!(env!("OUT_DIR"), "/ops.rs"));

#[derive(Debug)]
struct JumpToResolve {
    target: JumpTarget,
    from: usize,
    size: Size,
}

impl Assembler {
    pub fn new() -> Assembler {
        Assembler{
            inner: FlaggedAssembler{
                buffer: Vec::new(),
                prefixes: Vec::new(),
                jump_target_counter: 5050,
                allocated_jump_targets: HashSet::new(),
            },
        }
    }

    pub fn global(&mut self, name: &str) {
        self.inner.buffer.push(Stmt::GlobalLabel(name.to_string()));
    }
    
    pub fn constant(&mut self, xs: &[u8]) {
        for x in xs {
            self.inner.buffer.push(Stmt::Const(*x));
        }
    }
    
    pub fn allocate_local(&mut self) -> JumpTarget {
        loop {
            let ret = self.jump_target_counter;
            
            // I want this to be deterministic but not terribly likely to be duplicated between different Assembler instances
            self.jump_target_counter = self.jump_target_counter.wrapping_add(47).wrapping_add(self.buffer.len() as u64).wrapping_mul(199);
            
            if !self.allocated_jump_targets.contains(&ret) {
                self.allocated_jump_targets.insert(ret);
                return ret;
            }
        }
    }
    
    pub fn place_local(&mut self, target: JumpTarget) {
        self.inner.buffer.push(Stmt::LocalLabel(target));
    }
    
    pub fn local(&mut self) -> JumpTarget {
        let ret = self.allocate_local();
        self.place_local(ret);
        ret
    }
    
    pub fn align(&mut self, alignment_bytes: u64) {
        self.inner.buffer.push(Stmt::Align(ImmediateValue::U64(alignment_bytes)));
    }
    
    pub fn dump(&self) -> ObjectFile {
        let mut result = ObjectFile{
            code: Vec::new(),
            functions: Vec::new(),
        };
        
        let mut labels = HashMap::new();
        let mut jumps_to_resolve = Vec::new();
        
        for stmt in &self.inner.buffer {
            println!("{:?}", stmt);
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
                Stmt::Var(ImmediateValue::U64(x), Size::QWORD) => {
                    let mut xs = [0u8; 8];
                    LittleEndian::write_u64(&mut xs[..], x);
                    result.code.extend(xs.iter());
                }
                Stmt::GlobalLabel(ref ident) => {
                    result.functions.push(ExportedFunction{
                        offset: result.code.len() as u32,
                        name: ident.clone(),
                    });
                }
                Stmt::LocalLabel(target) => {
                    labels.insert(target, result.code.len());
                }
                Stmt::ForwardJumpTarget(target, size) => {
                    jumps_to_resolve.push(JumpToResolve{
                        target: target,
                        size: size,
                        from: result.code.len(),
                    });
                }
                Stmt::Align(ImmediateValue::U64(x)) => {
                    if x > 1024 {
                        panic!("Excessive alignment request: {}", x);
                    }
                    let x = x as usize;
                    while result.code.len() % x != 0 {
                        result.code.push(0x90);
                    }
                }
                _ => { panic!("Unimplemented statement: {:?}", stmt); }
            }
        }
        
        println!("Jumps = {:?}", jumps_to_resolve);
        
        for jump_to_resolve in jumps_to_resolve {
            let target_addr = match labels.get(&jump_to_resolve.target) {
                Some(target_addr) => *target_addr,
                None => panic!("Unresolved address: {}", jump_to_resolve.target)
            };
            
            let jump_amount = (target_addr as i32) - ((jump_to_resolve.from) as i32);
            match jump_to_resolve.size {
                Size::DWORD => {
                    LittleEndian::write_i32(&mut result.code[jump_to_resolve.from - 4..], jump_amount);
                },
                _ => {
                    panic!("Unimplemented jump size")
                }
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
