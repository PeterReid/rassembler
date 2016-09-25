extern crate byteorder;
extern crate coff_writer;
extern crate elf_writer;

mod object_file;
pub mod x64;

pub enum Arch {
    X64(x64::Assembler)
}

#[cfg(target_arch = "x86_64")]
pub fn new(object_name: &str) -> Arch {
    Arch::X64(x64::Assembler::new())
}

#[test]
fn it_works() {
}
