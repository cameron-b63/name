/// This enum contains all the "registers" that are a part of the coprocessor 0 definition, even though the majority will go unused.
/// It is based on this [specification](https://s3-eu-west-1.amazonaws.com/downloads-mips/documents/MD00090-2B-MIPS32PRA-AFP-06.02.pdf).
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Register {
    // Register 0
    Index,
    MVPControl,
    MVPConf0,
    MVPConf1,
    // Register 1
    Random,
    VPEControl,
    VPEConf0,
    VPEConf1,
    YQMask,
    VPESchedule,
    VPEScheFBack,
    VPEOpt,
    // Register 2
    EntryLo0,
    TCStatus,
    TCBind,
    TCRestart,
    TCHalt,
    TCContext,
    TCSchedule,
    TCScheFBack,
    // Register 3
    EntryLo1,
    TCOpt,
    // Register 4
    Context,
    ContextConfig,
    UserLocal,
    // Reserved
    // Register 5
    PageMask,
    PageGrain,
    SegCtl0,
    SegCtl1,
    SegCtl2,
    PWBase,
    PWField,
    PWSize,
    // Register 6
    Wired,
    SRSConf0,
    SRSConf1,
    SRSConf2,
    SRSConf3,
    SRSConf4,
    PWCtl,
    // Register 7
    HWREna,
    // Reserved
    // Register 8
    BadVAddr,
    BadInstr,
    BadInstrP,
    // Register 9
    Count,
    // Implementation-Dependent
    // Register 10
    EntryHi,
    GuestCtl1,
    GuestCtl2,
    GuestCtl3,
    // Register 11
    Compare,
    GuestCtl0Ext,
    // Implementation-Dependent
    // Register 12
    Status,
    IntCtl,
    SRSCtl,
    SRSMap,
    ViewIPL,
    SRSMap2,
    GuestCtl0,
    GTOffset,
    // Register 13
    Cause,
    ViewRIPL,
    NestedEcx,
    // Register 14
    EPC,
    NestedEPC,
    // Register 15
    PRId,
    EBase,
    CDMMBase,
    CMGCRBase,
    // Register 16
    Config,
    Config1,
    Config2,
    Config3,
    Config4,
    Config5,
    // Implementation-Dependent
    // Register 17
    LLAddr,
    // Register 18
    WatchLo,
    // Register 19
    WatchHi,
    // Register 20
    // Reserved
    // Register 21
    // Reserved
    // Register 22
    // Implementation-Dependent
    // Register 23
    Debug,
    TraceControl,
    TraceControl2,
    UserTraceData1,
    TraceIBPC,
    TraceDBPC,
    Debug2,
    // Register 24
    DEPC,
    TraceControl3,
    UserTraceData2,
    // Register 25
    PerfCnt,
    // Register 26
    ErrCtl,
    // Register 27
    CacheErr,
    // Register 28
    TagLo,
    DataLo,
    // Register 29
    TagHi,
    DataHi,
    // Register 30
    ErrorEPC,
    // Register 31
    DESAVE,
    KScratchn,
}
