use parser::{Register, Size, RegId, RegKind, Arg};

/*            "ymm0"  => (XMM0 , HWORD), "ymm1"  => (XMM1 , HWORD), "ymm2"  => (XMM2 , HWORD), "ymm3"  => (XMM3 , HWORD),
            "ymm4"  => (XMM4 , HWORD), "ymm5"  => (XMM5 , HWORD), "ymm6"  => (XMM6 , HWORD), "ymm7"  => (XMM7 , HWORD),
            "ymm8"  => (XMM8 , HWORD), "ymm9"  => (XMM9 , HWORD), "ymm10" => (XMM10, HWORD), "ymm11" => (XMM11, HWORD),
            "ymm12" => (XMM12, HWORD), "ymm13" => (XMM13, HWORD), "ymm14" => (XMM14, HWORD), "ymm15" => (XMM15, HWORD),
*/

macro_rules! reg_enum {
    ( $name:ident = [
        $(
            $case_name:ident => $reg_id:ident $size:ident ;
        )*
    ]
      
    ) => {
        #[derive(Debug, Copy, Clone)]
        pub enum $name {
            $($case_name),*
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
    HWord = [
        Ymm0 => XMM0 HWORD;
        Ymm1 => XMM1 HWORD;
        Ymm2 => XMM2 HWORD;
        Ymm3 => XMM3 HWORD;
        Ymm4 => XMM4 HWORD;
        Ymm5 => XMM5 HWORD;
        Ymm6 => XMM6 HWORD;
        Ymm7 => XMM7 HWORD;
        Ymm8 => XMM8 HWORD;
        Ymm9 => XMM9 HWORD;
        Ymm10 => XMM10 HWORD;
        Ymm11 => XMM11 HWORD;
        Ymm12 => XMM12 HWORD;
        Ymm13 => XMM13 HWORD;
        Ymm14 => XMM14 HWORD;
        Ymm15 => XMM15 HWORD;
    ]
}


