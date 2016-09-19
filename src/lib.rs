extern crate byteorder;
extern crate coff_writer;
extern crate elf_writer;

mod assembler;
pub mod parser;
mod x64data;
mod compiler;

pub use assembler::{Assembler, FlaggedAssembler}; 

#[test]
fn it_works() {
}
