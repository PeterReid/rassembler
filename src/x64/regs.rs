use x64::parser::{Register, MemoryRef, ImmediateValue, Size, RegId, RegKind, Arg, JumpType, JumpTarget};

macro_rules! reg_enum {
    ( $name:ident: $size:ident = [
        $(
            $case_name:ident => $reg_id:ident ;
        )*
    ]
      
    ) => {
        #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
        pub enum $name {
            $($case_name),*
        }
        
        impl $name {
            pub fn from_index(index: usize) -> $name {
                $(
                    if RegId::$reg_id.code() as usize == index { return $name::$case_name; }
                )*
                panic!("Invalid index for register")
            }
        }
        
        impl Into<Register> for $name {
            fn into(self) -> Register {
                match self {
                    $(
                        $name::$case_name => Register{size: Size::$size, kind: RegKind::Static(RegId::$reg_id)},
                    )*
                }
            }
        }
        
        impl Into<Arg> for $name {
            fn into(self) -> Arg {
                Arg::Direct(self.into())
            }
        }
    }
}

reg_enum! {
    Byte: BYTE = [
        Al => RAX;
        Cl => RCX;
        Dl => RDX;
        Bl => RBX;
        Spl => RSP;
        Rpl => RBP;
        Sil => RSI;
        Dil => RDI;
        
        R0b => RAX;
        R1b => RCX;
        R2b => RDX;
        R3b => RBX;
        
        R8b => R8;
        R9b => R9;
        R10b => R10;
        R11b => R11;
        R12b => R12;
        R13b => R13;
        R14b => R14;
        R15b => R15;
    
    ]
}

reg_enum! {
    DWord: DWORD = [
        Eax => RAX;
        Ecx => RCX;
        Edx => RDX;
        Ebx => RBX;
        Esp => RSP;
        Ebp => RBP;
        Esi => RSI;
        Edi => RDI;
        
        R0d => RAX;
        R1d => RCX;
        R2d => RDX;
        R3d => RBX;
        R4d => RSP;
        R5d => RBP;
        R6d => RSI;
        R7d => RDI;
        R8d => R8;
        R9d => R9;
        R10d => R10;
        R11d => R11;
        R12d => R12;
        R13d => R13;
        R14d => R14;
        R15d => R15;
    ]
}
            
reg_enum! {
    QWord: QWORD = [
        Rax => RAX;
        Rcx => RCX;
        Rdx => RDX;
        Rbx => RBX;
        Rsp => RSP;
        Rbp => RBP;
        Rsi => RSI;
        Rdi => RDI;
        
        R0 => RAX;
        R1 => RCX;
        R2 => RDX;
        R3 => RBX;
        R4 => RSP;
        R5 => RBP;
        R6 => RSI;
        R7 => RDI;
        R8 => R8;
        R9 => R9;
        R10 => R10;
        R11 => R11;
        R12 => R12;
        R13 => R13;
        R14 => R14;
        R15 => R15;
    ]
}

impl QWord {
    pub fn value_at(self) -> Arg {
        self.value_at_offset(0)
    }
    
    pub fn value_at_offset(self, offset: i32) -> Arg {
        Arg::Indirect(MemoryRef{
            index: None,
            scale: 0,
            base: Some(self.into()),
            disp: if offset != 0 {
                    Some(ImmediateValue::I64(offset as i64))
                } else {
                    None
                },
            size: None,
        })
    }
}

reg_enum! {
    OWord: OWORD = [
        Xmm0 => XMM0;
        Xmm1 => XMM1;
        Xmm2 => XMM2;
        Xmm3 => XMM3;
        Xmm4 => XMM4;
        Xmm5 => XMM5;
        Xmm6 => XMM6;
        Xmm7 => XMM7;
        Xmm8 => XMM8;
        Xmm9 => XMM9;
        Xmm10 => XMM10;
        Xmm11 => XMM11;
        Xmm12 => XMM12;
        Xmm13 => XMM13;
        Xmm14 => XMM14;
        Xmm15 => XMM15;
    ]
}

reg_enum! {
    HWord: HWORD = [
        Ymm0 => XMM0;
        Ymm1 => XMM1;
        Ymm2 => XMM2;
        Ymm3 => XMM3;
        Ymm4 => XMM4;
        Ymm5 => XMM5;
        Ymm6 => XMM6;
        Ymm7 => XMM7;
        Ymm8 => XMM8;
        Ymm9 => XMM9;
        Ymm10 => XMM10;
        Ymm11 => XMM11;
        Ymm12 => XMM12;
        Ymm13 => XMM13;
        Ymm14 => XMM14;
        Ymm15 => XMM15;
    ]
}

pub fn rip_relative(target: JumpTarget) -> Arg { 
    Arg::IndirectJumpTarget(JumpType::Forward(target), None)
}
pub fn rip_nonrelative(label: JumpTarget) -> Arg { 
    Arg::JumpTarget(JumpType::Forward(label), None)
}