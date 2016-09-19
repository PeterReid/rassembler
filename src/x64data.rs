pub mod flags {
    pub const VEX_OP    : u32 = 0x0000_0001; // this instruction requires a VEX prefix to be encoded
    pub const XOP_OP    : u32 = 0x0000_0002; // this instruction requires a XOP prefix to be encoded

    // note: the first 4 in this block are mutually exclusive
    pub const AUTO_SIZE : u32 = 0x0000_0004; // 16 bit -> OPSIZE , 32-bit -> None   , 64-bit -> REX.W/VEX.W/XOP.W
    pub const AUTO_NO32 : u32 = 0x0000_0008; // 16 bit -> OPSIZE , 32-bit -> illegal, 64-bit -> None
    pub const AUTO_REXW : u32 = 0x0000_0010; // 16 bit -> illegal, 32-bit -> None   , 64-bit -> REX.W/VEX.W/XOP.W
    pub const AUTO_VEXL : u32 = 0x0000_0020; // 128bit -> None   , 256bit -> VEX.L
    pub const WORD_SIZE : u32 = 0x0000_0040; // implies opsize prefix
    pub const WITH_REXW : u32 = 0x0000_0080; // implies REX.W/VEX.W/XOP.W
    pub const WITH_VEXL : u32 = 0x0000_0100; // implies VEX.L/XOP.L

    pub const PREF_66   : u32 = WORD_SIZE; // mandatory prefix (same as WORD_SIZE)
    pub const PREF_67   : u32 = 0x0000_0200; // mandatory prefix (same as SMALL_ADDRESS)
    pub const PREF_F0   : u32 = 0x0000_0400; // mandatory prefix (same as LOCK)
    pub const PREF_F2   : u32 = 0x0000_0800; // mandatory prefix (REPNE)
    pub const PREF_F3   : u32 = 0x0000_1000; // mandatory prefix (REP)

    pub const LOCK      : u32 = 0x0000_2000; // user lock prefix is valid with this instruction
    pub const REP       : u32 = 0x0000_4000; // user rep prefix is valid with this instruction
    pub const REPE      : u32 = 0x0000_8000;

    pub const SHORT_ARG : u32 = 0x0001_0000; // a register argument is encoded in the last byte of the opcode
    pub const ENC_MR    : u32 = 0x0002_0000; //  select alternate arg encoding
    pub const ENC_VM    : u32 = 0x0004_0000; //  select alternate arg encoding
}

