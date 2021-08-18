

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