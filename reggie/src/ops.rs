use super::ids::{
    ArgumentRegisterID, GlobalCellID, JmpLabel, LocalBlockID, LocalRegisterID, StringID,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Instruction {
    // lda_XYZ
    LdaRF(ArgumentRegisterID),
    LdaRS(ArgumentRegisterID),
    LdaRI(ArgumentRegisterID),
    LdaRT(ArgumentRegisterID),
    LdaRC(ArgumentRegisterID),
    LdaRU(ArgumentRegisterID),
    LdaRD(ArgumentRegisterID),

    LdaLF(LocalRegisterID),
    LdaLS(LocalRegisterID),
    LdaLI(LocalRegisterID),
    LdaLT(LocalRegisterID),
    LdaLC(LocalRegisterID),
    LdaLU(LocalRegisterID),
    LdaLD(LocalRegisterID),

    // str_XYZ
    StrRF(ArgumentRegisterID),
    StrRS(ArgumentRegisterID),
    StrRI(ArgumentRegisterID),
    StrRT(ArgumentRegisterID),
    StrRC(ArgumentRegisterID),
    StrRU(ArgumentRegisterID),
    StrRD(ArgumentRegisterID),

    StrLF(LocalRegisterID),
    StrLS(LocalRegisterID),
    StrLI(LocalRegisterID),
    StrLT(LocalRegisterID),
    StrLC(LocalRegisterID),
    StrLU(LocalRegisterID),
    StrLD(LocalRegisterID),

    // mov_ABC_XBZ - is not really needed RN

    // lda_X_gl
    LdaFGl(GlobalCellID),
    LdaIGl(GlobalCellID),
    LdaSGl(GlobalCellID),
    LdaTGl(GlobalCellID),
    LdaCGl(GlobalCellID),
    LdaUGl(GlobalCellID),
    LdaDGl(GlobalCellID),

    // str_X_gl
    StrFGl(GlobalCellID),
    StrIGl(GlobalCellID),
    StrSGl(GlobalCellID),
    StrTGl(GlobalCellID),
    StrCGl(GlobalCellID),
    StrUGl(GlobalCellID),
    StrDGl(GlobalCellID),

    // lda_dyn_gl
    LdaDynGl,

    // str_dyn_gl
    StrDynGl,

    // lda_prot_Z
    LdaProt(ArgumentRegisterID),

    // RX_shift_right
    RFShiftRight,
    RIShiftRight,
    RSShiftRight,
    RTShiftRight,
    RCShiftRight,
    RUShiftRight,
    RDShiftRight,

    // F_add_XZ
    FAddR(ArgumentRegisterID),
    FAddL(LocalRegisterID),

    // F_mul_XZ
    FMulR(ArgumentRegisterID),
    FMulL(LocalRegisterID),

    // F_sub_XZ
    FSubR(ArgumentRegisterID),
    FSubL(LocalRegisterID),

    // F_div_XZ
    FDivR(ArgumentRegisterID),
    FDivL(LocalRegisterID),

    // I_add_XZ
    IAddR(ArgumentRegisterID),
    IAddL(LocalRegisterID),

    // I_mul_XZ
    IMulR(ArgumentRegisterID),
    IMulL(LocalRegisterID),

    // I_sub_XZ
    ISubR(ArgumentRegisterID),
    ISubL(LocalRegisterID),

    // I_div_XZ
    IDivR(ArgumentRegisterID),
    IDivL(LocalRegisterID),

    // D_add_XZ
    DAddR(ArgumentRegisterID),
    DAddL(LocalRegisterID),

    // D_mul_XZ
    DMulR(ArgumentRegisterID),
    DMulL(LocalRegisterID),

    // D_sub_XZ
    DSubR(ArgumentRegisterID),
    DSubL(LocalRegisterID),

    // D_div_XZ
    DDivR(ArgumentRegisterID),
    DDivL(LocalRegisterID),

    // S_concat_XZ
    SConcatR(ArgumentRegisterID),
    SConcatL(ArgumentRegisterID),

    // D_concat_XZ
    DConcatR(ArgumentRegisterID),
    DConcatL(ArgumentRegisterID),

    // I_to_s
    IToS,
    // F_to_s
    FToS,
    // D_to_s
    DToS,

    // assoc_XDZ
    AssocRD(ArgumentRegisterID),
    AssocLD(LocalRegisterID),
    // assoc_ASD
    AssocASD,

    // lda_assoc
    LdaAssocAD,
    LdaAssocAS,

    // push_D
    PushD,

    // str_vc
    StrVC,
    // lda_vc
    LdaVC,
    // call
    Call,
    // typed_call
    TypedCall,
    // D_call
    DCall,
    // ret
    Ret,

    // eq_test_XYZ
    EqTestRF(ArgumentRegisterID),
    EqTestRS(ArgumentRegisterID),
    EqTestRI(ArgumentRegisterID),
    EqTestRT(ArgumentRegisterID),
    EqTestRC(ArgumentRegisterID),
    EqTestRU(ArgumentRegisterID),
    EqTestRD(ArgumentRegisterID),

    EqTestLF(LocalRegisterID),
    EqTestLS(LocalRegisterID),
    EqTestLI(LocalRegisterID),
    EqTestLT(LocalRegisterID),
    EqTestLC(LocalRegisterID),
    EqTestLU(LocalRegisterID),
    EqTestLD(LocalRegisterID),

    // test_XYZ
    TestRF(ArgumentRegisterID),
    TestRS(ArgumentRegisterID),
    TestRI(ArgumentRegisterID),
    TestRT(ArgumentRegisterID),
    TestRC(ArgumentRegisterID),
    TestRU(ArgumentRegisterID),
    TestRD(ArgumentRegisterID),

    TestLF(LocalRegisterID),
    TestLS(LocalRegisterID),
    TestLI(LocalRegisterID),
    TestLT(LocalRegisterID),
    TestLC(LocalRegisterID),
    TestLU(LocalRegisterID),
    TestLD(LocalRegisterID),

    // type_test
    TypeTest,

    // nil_test
    NilTest,

    // const_F
    ConstF(f64),
    // const_I
    ConstI(i32),
    // const_N
    ConstN,
    // const_S
    ConstS(StringID),
    // const_C
    ConstC(LocalBlockID),

    // new_T
    NewT,

    // wrap_X
    WrapF,
    WrapI,
    WrapS,
    WrapC,
    WrapT,
    WrapU,

    // cast_X
    CastF,
    CastI,
    CastS,
    CastC,
    CastT,
    CastU,

    // label
    Label,

    // jmp
    Jmp(JmpLabel),
    // jmplt
    JmpLT(JmpLabel),
    // jmpgt
    JmpGT(JmpLabel),
    // jmpeq
    JmpEQ(JmpLabel),
    // jmpne
    JmpNE(JmpLabel),
    // jmple
    JmpLE(JmpLabel),
    // jmpge
    JmpGE(JmpLabel),
    // jmpN
    JmpN(JmpLabel),
    // jmpF
    JmpF(JmpLabel),
    // jmpI
    JmpI(JmpLabel),
    // jmpC
    JmpC(JmpLabel),
    // jmpT
    JmpT(JmpLabel),
    // jmpU
    JmpU(JmpLabel),

    // errors
    TablePropertyLookupError,
    TableMemberLookupErrorR(ArgumentRegisterID),
    TableMemberLookupErrorL(LocalRegisterID),
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Instruction::LdaRF(reg) => write!(f, "lda RF{}", reg.0),
            Instruction::LdaRS(reg) => write!(f, "lda RS{}", reg.0),
            Instruction::LdaRI(reg) => write!(f, "lda RI{}", reg.0),
            Instruction::LdaRT(reg) => write!(f, "lda RT{}", reg.0),
            Instruction::LdaRC(reg) => write!(f, "lda RC{}", reg.0),
            Instruction::LdaRU(reg) => write!(f, "lda RU{}", reg.0),
            Instruction::LdaRD(reg) => write!(f, "lda RD{}", reg.0),
            Instruction::LdaLF(reg) => write!(f, "lda LF{}", reg.0),
            Instruction::LdaLS(reg) => write!(f, "lda LS{}", reg.0),
            Instruction::LdaLI(reg) => write!(f, "lda LI{}", reg.0),
            Instruction::LdaLT(reg) => write!(f, "lda LT{}", reg.0),
            Instruction::LdaLC(reg) => write!(f, "lda LC{}", reg.0),
            Instruction::LdaLU(reg) => write!(f, "lda LU{}", reg.0),
            Instruction::LdaLD(reg) => write!(f, "lda LD{}", reg.0),
            Instruction::StrRF(reg) => write!(f, "str RF{}", reg.0),
            Instruction::StrRS(reg) => write!(f, "str RS{}", reg.0),
            Instruction::StrRI(reg) => write!(f, "str RI{}", reg.0),
            Instruction::StrRT(reg) => write!(f, "str RT{}", reg.0),
            Instruction::StrRC(reg) => write!(f, "str RC{}", reg.0),
            Instruction::StrRU(reg) => write!(f, "str RU{}", reg.0),
            Instruction::StrRD(reg) => write!(f, "str RD{}", reg.0),
            Instruction::StrLF(reg) => write!(f, "str LF{}", reg.0),
            Instruction::StrLS(reg) => write!(f, "str LS{}", reg.0),
            Instruction::StrLI(reg) => write!(f, "str LI{}", reg.0),
            Instruction::StrLT(reg) => write!(f, "str LT{}", reg.0),
            Instruction::StrLC(reg) => write!(f, "str LC{}", reg.0),
            Instruction::StrLU(reg) => write!(f, "str LU{}", reg.0),
            Instruction::StrLD(reg) => write!(f, "str LD{}", reg.0),
            Instruction::LdaFGl(cell) => write!(f, "lda_F_gl {}", cell.0),
            Instruction::LdaIGl(cell) => write!(f, "lda_I_gl {}", cell.0),
            Instruction::LdaSGl(cell) => write!(f, "lda_S_gl {}", cell.0),
            Instruction::LdaTGl(cell) => write!(f, "lda_T_gl {}", cell.0),
            Instruction::LdaCGl(cell) => write!(f, "lda_C_gl {}", cell.0),
            Instruction::LdaUGl(cell) => write!(f, "lda_U_gl {}", cell.0),
            Instruction::LdaDGl(cell) => write!(f, "lda_D_gl {}", cell.0),
            Instruction::StrFGl(cell) => write!(f, "str_F_gl {}", cell.0),
            Instruction::StrIGl(cell) => write!(f, "str_I_gl {}", cell.0),
            Instruction::StrSGl(cell) => write!(f, "str_S_gl {}", cell.0),
            Instruction::StrTGl(cell) => write!(f, "str_T_gl {}", cell.0),
            Instruction::StrCGl(cell) => write!(f, "str_C_gl {}", cell.0),
            Instruction::StrUGl(cell) => write!(f, "str_U_gl {}", cell.0),
            Instruction::StrDGl(cell) => write!(f, "str_D_gl {}", cell.0),
            Instruction::LdaDynGl => write!(f, "lda_dyn_gl"),
            Instruction::StrDynGl => write!(f, "str_dyn_gl"),
            Instruction::LdaProt(reg) => write!(f, "lda_prot_{}", reg.0),
            Instruction::RFShiftRight => write!(f, "RF_shift_right"),
            Instruction::RIShiftRight => write!(f, "RI_shift_right"),
            Instruction::RSShiftRight => write!(f, "RS_shift_right"),
            Instruction::RTShiftRight => write!(f, "RT_shift_right"),
            Instruction::RCShiftRight => write!(f, "RC_shift_right"),
            Instruction::RUShiftRight => write!(f, "RU_shift_right"),
            Instruction::RDShiftRight => write!(f, "RD_shift_right"),
            Instruction::FAddR(reg) => write!(f, "add RF{}", reg.0),
            Instruction::FAddL(reg) => write!(f, "add LF{}", reg.0),
            Instruction::FMulR(reg) => write!(f, "mul RF{}", reg.0),
            Instruction::FMulL(reg) => write!(f, "mul LF{}", reg.0),
            Instruction::FSubR(reg) => write!(f, "sub RF{}", reg.0),
            Instruction::FSubL(reg) => write!(f, "sub LF{}", reg.0),
            Instruction::FDivR(reg) => write!(f, "div RF{}", reg.0),
            Instruction::FDivL(reg) => write!(f, "div LF{}", reg.0),
            Instruction::IAddR(reg) => write!(f, "add RI{}", reg.0),
            Instruction::IAddL(reg) => write!(f, "add LI{}", reg.0),
            Instruction::IMulR(reg) => write!(f, "mul RI{}", reg.0),
            Instruction::IMulL(reg) => write!(f, "mul LI{}", reg.0),
            Instruction::ISubR(reg) => write!(f, "sub RI{}", reg.0),
            Instruction::ISubL(reg) => write!(f, "sub LI{}", reg.0),
            Instruction::IDivR(reg) => write!(f, "div RI{}", reg.0),
            Instruction::IDivL(reg) => write!(f, "div LI{}", reg.0),
            Instruction::DAddR(reg) => write!(f, "add RD{}", reg.0),
            Instruction::DAddL(reg) => write!(f, "add LD{}", reg.0),
            Instruction::DMulR(reg) => write!(f, "mul RD{}", reg.0),
            Instruction::DMulL(reg) => write!(f, "mul LD{}", reg.0),
            Instruction::DSubR(reg) => write!(f, "sub RD{}", reg.0),
            Instruction::DSubL(reg) => write!(f, "sub LD{}", reg.0),
            Instruction::DDivR(reg) => write!(f, "div RD{}", reg.0),
            Instruction::DDivL(reg) => write!(f, "div LD{}", reg.0),
            Instruction::SConcatR(reg) => write!(f, "concat RS{}", reg.0),
            Instruction::SConcatL(reg) => write!(f, "concat LS{}", reg.0),
            Instruction::DConcatR(reg) => write!(f, "concat RD{}", reg.0),
            Instruction::DConcatL(reg) => write!(f, "concat LD{}", reg.0),
            Instruction::IToS => write!(f, "I_to_s"),
            Instruction::FToS => write!(f, "F_to_s"),
            Instruction::DToS => write!(f, "D_to_s"),
            Instruction::StrVC => write!(f, "std_vc"),
            Instruction::LdaVC => write!(f, "lda_vc"),
            Instruction::Call => write!(f, "call"),
            Instruction::TypedCall => write!(f, "typed_call"),
            Instruction::DCall => write!(f, "D_call"),
            Instruction::Ret => write!(f, "ret"),
            Instruction::EqTestRF(reg) => write!(f, "eq_test RF{}", reg.0),
            Instruction::EqTestRS(reg) => write!(f, "eq_test RS{}", reg.0),
            Instruction::EqTestRI(reg) => write!(f, "eq_test RI{}", reg.0),
            Instruction::EqTestRT(reg) => write!(f, "eq_test RT{}", reg.0),
            Instruction::EqTestRC(reg) => write!(f, "eq_test RC{}", reg.0),
            Instruction::EqTestRU(reg) => write!(f, "eq_test RU{}", reg.0),
            Instruction::EqTestRD(reg) => write!(f, "eq_test RD{}", reg.0),
            Instruction::EqTestLF(reg) => write!(f, "eq_test LF{}", reg.0),
            Instruction::EqTestLS(reg) => write!(f, "eq_test LS{}", reg.0),
            Instruction::EqTestLI(reg) => write!(f, "eq_test LI{}", reg.0),
            Instruction::EqTestLT(reg) => write!(f, "eq_test LT{}", reg.0),
            Instruction::EqTestLC(reg) => write!(f, "eq_test LC{}", reg.0),
            Instruction::EqTestLU(reg) => write!(f, "eq_test LU{}", reg.0),
            Instruction::EqTestLD(reg) => write!(f, "eq_test LD{}", reg.0),
            Instruction::TestRF(reg) => write!(f, "test RX{}", reg.0),
            Instruction::TestRS(reg) => write!(f, "test RX{}", reg.0),
            Instruction::TestRI(reg) => write!(f, "test RX{}", reg.0),
            Instruction::TestRT(reg) => write!(f, "test RX{}", reg.0),
            Instruction::TestRC(reg) => write!(f, "test RX{}", reg.0),
            Instruction::TestRU(reg) => write!(f, "test RX{}", reg.0),
            Instruction::TestRD(reg) => write!(f, "test RX{}", reg.0),
            Instruction::TestLF(reg) => write!(f, "test RX{}", reg.0),
            Instruction::TestLS(reg) => write!(f, "test RX{}", reg.0),
            Instruction::TestLI(reg) => write!(f, "test RX{}", reg.0),
            Instruction::TestLT(reg) => write!(f, "test RX{}", reg.0),
            Instruction::TestLC(reg) => write!(f, "test RX{}", reg.0),
            Instruction::TestLU(reg) => write!(f, "test RX{}", reg.0),
            Instruction::TestLD(reg) => write!(f, "test RX{}", reg.0),
            Instruction::TypeTest => write!(f, "type_test"),
            Instruction::NilTest => write!(f, "nil_test"),
            Instruction::ConstF(float) => write!(f, "const_F {}", float),
            Instruction::ConstI(int) => write!(f, "const_I {}", int),
            Instruction::ConstN => write!(f, "const_N"),
            Instruction::ConstS(string_id) => write!(f, "const_S {}", string_id.0),
            Instruction::ConstC(local_block_id) => write!(f, "const_C {}", local_block_id.0),
            Instruction::WrapF => write!(f, "wrap_F"),
            Instruction::WrapI => write!(f, "wrap_I"),
            Instruction::WrapS => write!(f, "wrap_S"),
            Instruction::WrapC => write!(f, "wrap_C"),
            Instruction::WrapT => write!(f, "wrap_T"),
            Instruction::WrapU => write!(f, "wrap_U"),
            Instruction::CastF => write!(f, "cast_F"),
            Instruction::CastI => write!(f, "cast_I"),
            Instruction::CastS => write!(f, "cast_S"),
            Instruction::CastC => write!(f, "cast_C"),
            Instruction::CastT => write!(f, "cast_T"),
            Instruction::CastU => write!(f, "cast_U"),
            Instruction::Label => write!(f, "label"),
            Instruction::Jmp(lbl) => write!(f, "jmp {}", lbl.0),
            Instruction::JmpLT(lbl) => write!(f, "jmp_lt {}", lbl.0),
            Instruction::JmpGT(lbl) => write!(f, "jmp_gt {}", lbl.0),
            Instruction::JmpEQ(lbl) => write!(f, "jmp_eq {}", lbl.0),
            Instruction::JmpNE(lbl) => write!(f, "jmp_ne {}", lbl.0),
            Instruction::JmpLE(lbl) => write!(f, "jmp_le {}", lbl.0),
            Instruction::JmpGE(lbl) => write!(f, "jmp_ge {}", lbl.0),
            Instruction::JmpN(lbl) => write!(f, "jmp_F {}", lbl.0),
            Instruction::JmpF(lbl) => write!(f, "jmp_I {}", lbl.0),
            Instruction::JmpI(lbl) => write!(f, "jmp_S {}", lbl.0),
            Instruction::JmpC(lbl) => write!(f, "jmp_C {}", lbl.0),
            Instruction::JmpT(lbl) => write!(f, "jmp_T {}", lbl.0),
            Instruction::JmpU(lbl) => write!(f, "jmp_U {}", lbl.0),
            Instruction::AssocRD(reg) => write!(f, "assoc RD{}", reg.0),
            Instruction::AssocLD(reg) => write!(f, "assoc LD{}", reg.0),
            Instruction::NewT => write!(f, "new_T"),
            Instruction::PushD => write!(f, "push_D"),
            Instruction::AssocASD => write!(f, "assoc AS D"),
            Instruction::LdaAssocAD => write!(f, "lda_assoc AD"),
            Instruction::LdaAssocAS => write!(f, "lda_assoc AS"),
            Instruction::TablePropertyLookupError => write!(f, "error table_property_lookup"),
            Instruction::TableMemberLookupErrorR(reg) => {
                write!(f, "error table_member_lookup RD{}", reg.0)
            }
            Instruction::TableMemberLookupErrorL(reg) => {
                write!(f, "error table_member_lookup LD{}", reg.0)
            }
        }
    }
}
