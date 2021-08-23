use crate::definitions::*;

pub fn console_type_lut<'a>(kind: u8) -> Option<&'a str> {
    match kind {
        0x01u8 => Some("NES"),
        0x02u8 => Some("SNES"),
        0x03u8 => Some("N64"),
        0x04u8 => Some("GC"),
        0x05u8 => Some("GB"),
        0x06u8 => Some("GBC"),
        0x07u8 => Some("GBA"),
        0x08u8 => Some("Genesis"),
        0x09u8 => Some("A2600"),
        _ => None
    }
}

pub fn console_region_lut<'a>(kind: u8) -> Option<&'a str> {
    match kind {
        0x01u8 => Some("NTSC"),
        0x02u8 => Some("PAL"),
        _ => None
    }
}

pub fn memory_init_lut<'a>(kind: u8) -> Option<&'a str> {
    match kind {
        0x01u8 => Some("No initialization required"),
        0x02u8 => Some("Custom"),
        0x03u8 => Some("All 0x00"),
        0x04u8 => Some("All 0xFF"),
        0x05u8 => Some("00 00 00 00 FF FF FF FF (repeating)"),
        0x06u8 => Some("Random (implementation-dependent)"),
        _ => None
    }
}

pub fn transition_lut<'a>(kind: u8) -> Option<&'a str> {
    match kind {
        0x01u8 => Some("\"Soft\" Reset"),
        0x02u8 => Some("Power Reset"),
        0x03u8 => Some("Controller Swap"),
        _ => None
    }
}

pub fn key_spec_lut(key: [u8; 2]) -> Option<PacketSpec> {
    match key {
        CONSOLE_TYPE => Some(ConsoleType::default().get_packet_spec()),
        CONSOLE_REGION => Some(ConsoleRegion::default().get_packet_spec()),
        GAME_TITLE => Some(GameTitle::default().get_packet_spec()),
        AUTHOR => Some(Author::default().get_packet_spec()),
        CATEGORY => Some(Category::default().get_packet_spec()),
        EMULATOR_NAME => Some(EmulatorName::default().get_packet_spec()),
        EMULATOR_VERSION => Some(EmulatorVersion::default().get_packet_spec()),
        TAS_LAST_MODIFIED => Some(TASLastModified::default().get_packet_spec()),
        DUMP_LAST_MODIFIED => Some(DumpLastModified::default().get_packet_spec()),
        TOTAL_FRAMES => Some(TotalFrames::default().get_packet_spec()),
        RERECORDS => Some(Rerecords::default().get_packet_spec()),
        SOURCE_LINK => Some(SourceLink::default().get_packet_spec()),
        BLANK_FRAMES => Some(BlankFrames::default().get_packet_spec()),
        VERIFIED => Some(Verified::default().get_packet_spec()),
        MEMORY_INIT => Some(MemoryInit::default().get_packet_spec()),
        LATCH_FILTER => Some(LatchFilter::default().get_packet_spec()),
        CLOCK_FILTER => Some(ClockFilter::default().get_packet_spec()),
        OVERREAD => Some(Overread::default().get_packet_spec()),
        DPCM => Some(Dpcm::default().get_packet_spec()),
        GAME_GENIE_CODE => Some(GameGenieCode::default().get_packet_spec()),
        INPUT_CHUNKS => Some(InputChunks::default().get_packet_spec()),
        TRANSITION => Some(Transition::default().get_packet_spec()),
        LAG_FRAME_CHUNK => Some(LagFrameChunk::default().get_packet_spec()),
        _ => None
    }
}