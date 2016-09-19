extern crate byteorder;
extern crate coff_writer;
extern crate elf_writer;

mod assembler;
pub mod parser;
mod x64data;
mod compiler;
mod ops;

pub use ops::FlaggedAssembler;
pub use assembler::Assembler; 

#[test]
fn it_works() {
}
