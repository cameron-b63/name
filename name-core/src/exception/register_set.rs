use super::registers::Register;

/// Typed access to the CP0 register names declared in `registers.rs`.
///
/// The `(rd, sel)` mapping follows the MIPS32 PRA reference linked in
/// `exception/registers.rs`. A few registers represent families that can be
/// selected through multiple `sel` values (`WatchLo`, `WatchHi`, `PerfCnt`,
/// `KScratchn`). For those, `sel()` returns the lowest documented select and
/// `select_range()` reports the full valid range.
pub type Cp0Register = Register;

impl Register {
	pub const fn rd(self) -> usize {
		match self {
			Self::Index | Self::MVPControl | Self::MVPConf0 | Self::MVPConf1 => 0,
			Self::Random
			| Self::VPEControl
			| Self::VPEConf0
			| Self::VPEConf1
			| Self::YQMask
			| Self::VPESchedule
			| Self::VPEScheFBack
			| Self::VPEOpt => 1,
			Self::EntryLo0
			| Self::TCStatus
			| Self::TCBind
			| Self::TCRestart
			| Self::TCHalt
			| Self::TCContext
			| Self::TCSchedule
			| Self::TCScheFBack => 2,
			Self::EntryLo1 | Self::TCOpt => 3,
			Self::Context | Self::ContextConfig | Self::UserLocal => 4,
			Self::PageMask
			| Self::PageGrain
			| Self::SegCtl0
			| Self::SegCtl1
			| Self::SegCtl2
			| Self::PWBase
			| Self::PWField
			| Self::PWSize => 5,
			Self::Wired
			| Self::SRSConf0
			| Self::SRSConf1
			| Self::SRSConf2
			| Self::SRSConf3
			| Self::SRSConf4
			| Self::PWCtl => 6,
			Self::HWREna => 7,
			Self::BadVAddr | Self::BadInstr | Self::BadInstrP => 8,
			Self::Count => 9,
			Self::EntryHi | Self::GuestCtl1 | Self::GuestCtl2 | Self::GuestCtl3 => 10,
			Self::Compare | Self::GuestCtl0Ext => 11,
			Self::Status
			| Self::IntCtl
			| Self::SRSCtl
			| Self::SRSMap
			| Self::ViewIPL
			| Self::SRSMap2
			| Self::GuestCtl0
			| Self::GTOffset => 12,
			Self::Cause | Self::ViewRIPL | Self::NestedEcx => 13,
			Self::EPC | Self::NestedEPC => 14,
			Self::PRId | Self::EBase | Self::CDMMBase | Self::CMGCRBase => 15,
			Self::Config
			| Self::Config1
			| Self::Config2
			| Self::Config3
			| Self::Config4
			| Self::Config5 => 16,
			Self::LLAddr => 17,
			Self::WatchLo => 18,
			Self::WatchHi => 19,
			Self::Debug
			| Self::TraceControl
			| Self::TraceControl2
			| Self::UserTraceData1
			| Self::TraceIBPC
			| Self::TraceDBPC
			| Self::Debug2 => 23,
			Self::DEPC | Self::TraceControl3 | Self::UserTraceData2 => 24,
			Self::PerfCnt => 25,
			Self::ErrCtl => 26,
			Self::CacheErr => 27,
			Self::TagLo | Self::DataLo => 28,
			Self::TagHi | Self::DataHi => 29,
			Self::ErrorEPC => 30,
			Self::DESAVE | Self::KScratchn => 31,
		}
	}

	pub const fn sel(self) -> usize {
		match self {
			Self::Index => 0,
			Self::MVPControl => 1,
			Self::MVPConf0 => 2,
			Self::MVPConf1 => 3,
			Self::Random => 0,
			Self::VPEControl => 1,
			Self::VPEConf0 => 2,
			Self::VPEConf1 => 3,
			Self::YQMask => 4,
			Self::VPESchedule => 5,
			Self::VPEScheFBack => 6,
			Self::VPEOpt => 7,
			Self::EntryLo0 => 0,
			Self::TCStatus => 1,
			Self::TCBind => 2,
			Self::TCRestart => 3,
			Self::TCHalt => 4,
			Self::TCContext => 5,
			Self::TCSchedule => 6,
			Self::TCScheFBack => 7,
			Self::EntryLo1 => 0,
			Self::TCOpt => 7,
			Self::Context => 0,
			Self::ContextConfig => 1,
			Self::UserLocal => 2,
			Self::PageMask => 0,
			Self::PageGrain => 1,
			Self::SegCtl0 => 2,
			Self::SegCtl1 => 3,
			Self::SegCtl2 => 4,
			Self::PWBase => 5,
			Self::PWField => 6,
			Self::PWSize => 7,
			Self::Wired => 0,
			Self::SRSConf0 => 1,
			Self::SRSConf1 => 2,
			Self::SRSConf2 => 3,
			Self::SRSConf3 => 4,
			Self::SRSConf4 => 5,
			Self::PWCtl => 6,
			Self::HWREna => 0,
			Self::BadVAddr => 0,
			Self::BadInstr => 1,
			Self::BadInstrP => 2,
			Self::Count => 0,
			Self::EntryHi => 0,
			Self::GuestCtl1 => 1,
			Self::GuestCtl2 => 2,
			Self::GuestCtl3 => 3,
			Self::Compare => 0,
			Self::GuestCtl0Ext => 4,
			Self::Status => 0,
			Self::IntCtl => 1,
			Self::SRSCtl => 2,
			Self::SRSMap => 3,
			Self::ViewIPL => 4,
			Self::SRSMap2 => 5,
			Self::GuestCtl0 => 6,
			Self::GTOffset => 7,
			Self::Cause => 0,
			Self::ViewRIPL => 1,
			Self::NestedEcx => 2,
			Self::EPC => 0,
			Self::NestedEPC => 1,
			Self::PRId => 0,
			Self::EBase => 1,
			Self::CDMMBase => 2,
			Self::CMGCRBase => 3,
			Self::Config => 0,
			Self::Config1 => 1,
			Self::Config2 => 2,
			Self::Config3 => 3,
			Self::Config4 => 4,
			Self::Config5 => 5,
			Self::LLAddr => 0,
			Self::WatchLo => 0,
			Self::WatchHi => 0,
			Self::Debug => 0,
			Self::TraceControl => 1,
			Self::TraceControl2 => 2,
			Self::UserTraceData1 => 3,
			Self::TraceIBPC => 4,
			Self::TraceDBPC => 5,
			Self::Debug2 => 6,
			Self::DEPC => 0,
			Self::TraceControl3 => 1,
			Self::UserTraceData2 => 2,
			Self::PerfCnt => 0,
			Self::ErrCtl => 0,
			Self::CacheErr => 0,
			Self::TagLo => 0,
			Self::DataLo => 1,
			Self::TagHi => 0,
			Self::DataHi => 1,
			Self::ErrorEPC => 0,
			Self::DESAVE => 0,
			Self::KScratchn => 2,
		}
	}

	pub const fn index(self) -> (usize, usize) {
		(self.rd(), self.sel())
	}

	pub const fn select_range(self) -> (usize, usize) {
		match self {
			Self::WatchLo | Self::WatchHi | Self::PerfCnt => (0, 7),
			Self::KScratchn => (2, 7),
			_ => {
				let sel = self.sel();
				(sel, sel)
			}
		}
	}

	pub const fn is_valid_select(self, sel: usize) -> bool {
		let (min_sel, max_sel) = self.select_range();
		min_sel <= sel && sel <= max_sel
	}

	pub const fn reset_value(self) -> u32 {
		0
	}

	pub const fn write_mask(self) -> u32 {
		match self {
			Self::PRId => 0,
			Self::Cause => 0x00c0_0300,
			_ => u32::MAX,
		}
	}

	pub const fn from_index(rd: usize, sel: usize) -> Option<Self> {
		match (rd, sel) {
			(0, 0) => Some(Self::Index),
			(0, 1) => Some(Self::MVPControl),
			(0, 2) => Some(Self::MVPConf0),
			(0, 3) => Some(Self::MVPConf1),
			(1, 0) => Some(Self::Random),
			(1, 1) => Some(Self::VPEControl),
			(1, 2) => Some(Self::VPEConf0),
			(1, 3) => Some(Self::VPEConf1),
			(1, 4) => Some(Self::YQMask),
			(1, 5) => Some(Self::VPESchedule),
			(1, 6) => Some(Self::VPEScheFBack),
			(1, 7) => Some(Self::VPEOpt),
			(2, 0) => Some(Self::EntryLo0),
			(2, 1) => Some(Self::TCStatus),
			(2, 2) => Some(Self::TCBind),
			(2, 3) => Some(Self::TCRestart),
			(2, 4) => Some(Self::TCHalt),
			(2, 5) => Some(Self::TCContext),
			(2, 6) => Some(Self::TCSchedule),
			(2, 7) => Some(Self::TCScheFBack),
			(3, 0) => Some(Self::EntryLo1),
			(3, 7) => Some(Self::TCOpt),
			(4, 0) => Some(Self::Context),
			(4, 1) => Some(Self::ContextConfig),
			(4, 2) => Some(Self::UserLocal),
			(5, 0) => Some(Self::PageMask),
			(5, 1) => Some(Self::PageGrain),
			(5, 2) => Some(Self::SegCtl0),
			(5, 3) => Some(Self::SegCtl1),
			(5, 4) => Some(Self::SegCtl2),
			(5, 5) => Some(Self::PWBase),
			(5, 6) => Some(Self::PWField),
			(5, 7) => Some(Self::PWSize),
			(6, 0) => Some(Self::Wired),
			(6, 1) => Some(Self::SRSConf0),
			(6, 2) => Some(Self::SRSConf1),
			(6, 3) => Some(Self::SRSConf2),
			(6, 4) => Some(Self::SRSConf3),
			(6, 5) => Some(Self::SRSConf4),
			(6, 6) => Some(Self::PWCtl),
			(7, 0) => Some(Self::HWREna),
			(8, 0) => Some(Self::BadVAddr),
			(8, 1) => Some(Self::BadInstr),
			(8, 2) => Some(Self::BadInstrP),
			(9, 0) => Some(Self::Count),
			(10, 0) => Some(Self::EntryHi),
			(10, 1) => Some(Self::GuestCtl1),
			(10, 2) => Some(Self::GuestCtl2),
			(10, 3) => Some(Self::GuestCtl3),
			(11, 0) => Some(Self::Compare),
			(11, 4) => Some(Self::GuestCtl0Ext),
			(12, 0) => Some(Self::Status),
			(12, 1) => Some(Self::IntCtl),
			(12, 2) => Some(Self::SRSCtl),
			(12, 3) => Some(Self::SRSMap),
			(12, 4) => Some(Self::ViewIPL),
			(12, 5) => Some(Self::SRSMap2),
			(12, 6) => Some(Self::GuestCtl0),
			(12, 7) => Some(Self::GTOffset),
			(13, 0) => Some(Self::Cause),
			(13, 1) => Some(Self::ViewRIPL),
			(13, 2) => Some(Self::NestedEcx),
			(14, 0) => Some(Self::EPC),
			(14, 1) => Some(Self::NestedEPC),
			(15, 0) => Some(Self::PRId),
			(15, 1) => Some(Self::EBase),
			(15, 2) => Some(Self::CDMMBase),
			(15, 3) => Some(Self::CMGCRBase),
			(16, 0) => Some(Self::Config),
			(16, 1) => Some(Self::Config1),
			(16, 2) => Some(Self::Config2),
			(16, 3) => Some(Self::Config3),
			(16, 4) => Some(Self::Config4),
			(16, 5) => Some(Self::Config5),
			(17, 0) => Some(Self::LLAddr),
			(18, 0..=7) => Some(Self::WatchLo),
			(19, 0..=7) => Some(Self::WatchHi),
			(23, 0) => Some(Self::Debug),
			(23, 1) => Some(Self::TraceControl),
			(23, 2) => Some(Self::TraceControl2),
			(23, 3) => Some(Self::UserTraceData1),
			(23, 4) => Some(Self::TraceIBPC),
			(23, 5) => Some(Self::TraceDBPC),
			(23, 6) => Some(Self::Debug2),
			(24, 0) => Some(Self::DEPC),
			(24, 1) => Some(Self::TraceControl3),
			(24, 2) => Some(Self::UserTraceData2),
			(25, 0..=7) => Some(Self::PerfCnt),
			(26, 0) => Some(Self::ErrCtl),
			(27, 0) => Some(Self::CacheErr),
			(28, 0) => Some(Self::TagLo),
			(28, 1) => Some(Self::DataLo),
			(29, 0) => Some(Self::TagHi),
			(29, 1) => Some(Self::DataHi),
			(30, 0) => Some(Self::ErrorEPC),
			(31, 0) => Some(Self::DESAVE),
			(31, 2..=7) => Some(Self::KScratchn),
			_ => None,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::Cp0Register;

	#[test]
	fn key_cp0_register_mappings_match_documented_pairs() {
		assert_eq!(Cp0Register::BadVAddr.index(), (8, 0));
		assert_eq!(Cp0Register::Status.index(), (12, 0));
		assert_eq!(Cp0Register::Cause.index(), (13, 0));
		assert_eq!(Cp0Register::EBase.index(), (15, 1));
		assert_eq!(Cp0Register::Debug.index(), (23, 0));
		assert_eq!(Cp0Register::DEPC.index(), (24, 0));
		assert_eq!(Cp0Register::ErrorEPC.index(), (30, 0));
		assert_eq!(Cp0Register::KScratchn.select_range(), (2, 7));
	}

	#[test]
	fn cp0_family_registers_accept_documented_select_ranges() {
		assert!(Cp0Register::WatchLo.is_valid_select(0));
		assert!(Cp0Register::WatchLo.is_valid_select(7));
		assert!(Cp0Register::WatchHi.is_valid_select(3));
		assert!(Cp0Register::PerfCnt.is_valid_select(6));
		assert!(Cp0Register::KScratchn.is_valid_select(2));
		assert!(Cp0Register::KScratchn.is_valid_select(7));

		assert!(!Cp0Register::KScratchn.is_valid_select(1));
		assert!(!Cp0Register::Status.is_valid_select(1));
	}

	#[test]
	fn cp0_from_index_round_trips_known_entries() {
		for register in [
			Cp0Register::Index,
			Cp0Register::TCOpt,
			Cp0Register::BadInstr,
			Cp0Register::GuestCtl0Ext,
			Cp0Register::NestedEPC,
			Cp0Register::TraceControl3,
			Cp0Register::DataHi,
			Cp0Register::DESAVE,
		] {
			let (rd, sel) = register.index();
			assert_eq!(Cp0Register::from_index(rd, sel), Some(register));
		}

		assert_eq!(Cp0Register::from_index(18, 4), Some(Cp0Register::WatchLo));
		assert_eq!(Cp0Register::from_index(19, 7), Some(Cp0Register::WatchHi));
		assert_eq!(Cp0Register::from_index(31, 5), Some(Cp0Register::KScratchn));
	}
}
