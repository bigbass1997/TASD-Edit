use crate::definitions::*;

pub fn console_type_lut<'a>(kind: u8) -> Option<&'a str> {
    match kind {
        0x01 => Some("NES"),
        0x02 => Some("SNES"),
        0x03 => Some("N64"),
        0x04 => Some("GC"),
        0x05 => Some("GB"),
        0x06 => Some("GBC"),
        0x07 => Some("GBA"),
        0x08 => Some("Genesis"),
        0x09 => Some("A2600"),
        0xFF => Some("Custom"),
        _ => None
    }
}

pub fn console_region_lut<'a>(kind: u8) -> Option<&'a str> {
    match kind {
        0x01 => Some("NTSC"),
        0x02 => Some("PAL"),
        _ => None
    }
}

pub fn memory_init_lut<'a>(kind: u8) -> Option<&'a str> {
    match kind {
        0x01 => Some("No initialization required"),
        0x02 => Some("All 0x00"),
        0x03 => Some("All 0xFF"),
        0x04 => Some("00 00 00 00 FF FF FF FF (repeating)"),
        0x05 => Some("Random (implementation-dependent)"),
        0xFF => Some("Custom"),
        _ => None
    }
}

pub fn controller_type_lut<'a>(kind: u16) -> Option<&'a str> {
    match kind {
        0x0101 => Some("NES Standard"),
        0x0102 => Some("NES Multitap (Four Score)"),
        0x0103 => Some("NES Zapper"),
        0x0201 => Some("SNES Standard"),
        0x0202 => Some("SNES Multitap"),
        0x0203 => Some("SNES Mouse"),
        0x0204 => Some("SNES Superscope"),
        0x0301 => Some("N64 Standard"),
        0x0302 => Some("N64 Standard with Rumble Pak"),
        0x0303 => Some("N64 Standard with Controller Pak"),
        0x0304 => Some("N64 Standard with Transfer Pak"),
        0x0305 => Some("N64 Mouse"),
        0x0306 => Some("N64 Voice Recognition Unit (VRU)"),
        0x0307 => Some("N64 RandNet Keyboard"),
        0x0308 => Some("N64 Densha de Go"),
        0x0401 => Some("GC Standard"),
        0x0402 => Some("GC Keyboard"),
        0x0501 => Some("GB Gamepad"),
        0x0601 => Some("GBC Gamepad"),
        0x0701 => Some("GBA(SP) Gamepad"),
        0x0801 => Some("Genesis 3-Button"),
        0x0802 => Some("Genesis 6-Button"),
        0x0901 => Some("A2600 Joystick"),
        0x0902 => Some("A2600 Paddle"),
        0x0903 => Some("A2600 Keypad"),
        _ => None
    }
}

pub fn transition_lut<'a>(kind: u8) -> Option<&'a str> {
    match kind {
        0x01 => Some("\"Soft\" Reset"),
        0x02 => Some("Power Reset"),
        0xFF => Some("Packet-derived"),
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
        PORT_CONTROLLER => Some(PortController::default().get_packet_spec()),
        LATCH_FILTER => Some(LatchFilter::default().get_packet_spec()),
        CLOCK_FILTER => Some(ClockFilter::default().get_packet_spec()),
        OVERREAD => Some(Overread::default().get_packet_spec()),
        GAME_GENIE_CODE => Some(GameGenieCode::default().get_packet_spec()),
        INPUT_CHUNKS => Some(InputChunks::default().get_packet_spec()),
        TRANSITION => Some(Transition::default().get_packet_spec()),
        LAG_FRAME_CHUNK => Some(LagFrameChunk::default().get_packet_spec()),
        MOVIE_TRANSITION => Some(MovieTransition::default().get_packet_spec()),
        _ => None
    }
}