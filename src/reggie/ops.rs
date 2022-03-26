use super::ids::{ArgumentRegisterID, GlobalCellID, JmpLabel, LocalRegisterID, StringID};

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
    LdaDynGl(GlobalCellID),

    // str_dyn_gl
    StrDynGl(GlobalCellID),

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
    // set_vc
    SetVC,
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
    // panic not needed (and not spec'd) for now
}
