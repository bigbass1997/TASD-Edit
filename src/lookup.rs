
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

pub fn attribution_lut<'a>(kind: u8) -> Option<&'a str> {
    match kind {
        0x01 => Some("Author"),
        0x02 => Some("Verifier"),
        0x03 => Some("TASD File Creator"),
        0x04 => Some("TASD File Editor"),
        0xFF => Some("Other"),
        _ => None
    }
}

pub fn memory_init_lut<'a>(kind: u16) -> Option<&'a str> {
    match kind {
        0x0001 => Some("No initialization required"),
        0x0002 => Some("All 0x00"),
        0x0003 => Some("All 0xFF"),
        0x0004 => Some("00 00 00 00 FF FF FF FF (repeating)"),
        0x0005 => Some("Random"),
        0xFFFF => Some("Custom"),
        _ => None
    }
}

pub fn game_identifier_lut<'a>(kind: u8) -> Option<&'a str> {
    match kind {
        0x01 => Some("CRC-8 Checksum"),
        0x02 => Some("CRC-16 Checksum"),
        0x03 => Some("CRC-32 Checksum"),
        0x04 => Some("MD5 Hash"),
        0x05 => Some("SHA1 Hash"),
        0x06 => Some("SHA224 Hash"),
        0x07 => Some("SHA256 Hash"),
        0x08 => Some("SHA384 Hash"),
        0x09 => Some("SHA512 Hash"),
        0x0A => Some("SHA512/224 Hash"),
        0x0B => Some("SHA512/256 Hash"),
        0x0C => Some("SHA3-224 Hash"),
        0x0D => Some("SHA3-256 Hash"),
        0x0E => Some("SHA3-284 Hash"),
        0x0F => Some("SHAKE-128 Hash"),
        0x10 => Some("SHAKE-256 Hash"),
        0xFF => Some("Other"),
        _ => None
    }
}

pub fn controller_type_lut<'a>(kind: u16) -> Option<&'a str> {
    match kind {
        0x0101 => Some("NES Standard"),
        0x0102 => Some("NES Four Score"),
        0x0103 => Some("NES Zapper"),
        0x0201 => Some("SNES Standard"),
        0x0202 => Some("SNES Super Multitap"),
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
        0x0701 => Some("GBA Gamepad"),
        0x0801 => Some("Genesis (Mega Drive) 3-Button"),
        0x0802 => Some("Genesis (Mega Drive) 6-Button"),
        0x0901 => Some("A2600 Joystick"),
        0x0902 => Some("A2600 Paddle"),
        0x0903 => Some("A2600 Keypad"),
        _ => None
    }
}

pub fn input_moment_lut<'a>(kind: u8) -> Option<&'a str> {
    match kind {
        0x01 => Some("Frame"),
        0x02 => Some("Cycle Count"),
        0x03 => Some("Milliseconds"),
        0x04 => Some("Microseconds * 10"),
        _ => None
    }
}

pub fn transition_index_lut<'a>(kind: u8) -> Option<&'a str> {
    match kind {
        0x01 => Some("Frame"),
        0x02 => Some("Cycle Count"),
        0x03 => Some("Milliseconds"),
        0x04 => Some("Microseconds * 10"),
        0x05 => Some("INPUT_CHUNK Index"),
        _ => None
    }
}

pub fn transition_kind_lut<'a>(kind: u8) -> Option<&'a str> {
    match kind {
        0x01 => Some("\"Soft\" Reset"),
        0x02 => Some("Power Reset"),
        0x03 => Some("Restart TASD File"),
        0xFF => Some("Packet-derived"),
        _ => None
    }
}