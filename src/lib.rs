extern crate byteorder;
extern crate coff_writer;
extern crate elf_writer;

mod assembler;
pub mod parser;
mod x64data;
mod compiler;
mod regs;

pub use assembler::{Assembler, FlaggedAssembler}; 
pub use regs::HWord;

#[test]
fn it_works() {
}
