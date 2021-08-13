use byteorder::{ReadBytesExt, BigEndian};
use std::fmt::Display;
use strum_macros::EnumIter;
use std::time::Instant;
use chrono::{Utc, TimeZone};
use phf::phf_map;

macro_rules! key_const {
    ($name:ident, $upper:expr, $lower:expr) => {
        pub const $name: &[u8] = &[$upper, $lower];
    };
}

pub const MAGIC_NUMBER: &[u8] = &[0x54, 0x41, 0x53, 0x44];
pub const NEW_TASD_FILE: &[u8] = &[0x54, 0x41, 0x53, 0x44, 0x00, 0x01, 0x02];

key_const!(CONSOLE_TYPE, 0x00, 0x01);
key_const!(CONSOLE_REGION, 0x00, 0x02);
key_const!(GAME_TITLE, 0x00, 0x03);
key_const!(AUTHOR, 0x00, 0x04);
key_const!(CATEGORY, 0x00, 0x05);
key_const!(EMULATOR_NAME, 0x00, 0x06);
key_const!(EMULATOR_VERSION, 0x00, 0x07);
key_const!(TAS_LAST_MODIFIED, 0x00, 0x08);
key_const!(DUMP_LAST_MODIFIED, 0x00, 0x09);
key_const!(TOTAL_FRAMES, 0x00, 0x0A);
key_const!(RERECORDS, 0x00, 0x0B);
key_const!(SOURCE_LINK, 0x00, 0x0C);
key_const!(BLANK_FRAMES, 0x00, 0x0D);
key_const!(VERIFIED, 0x00, 0x0E);
key_const!(MEMORY_INIT, 0x00, 0x0F);

key_const!(LATCH_FILTER, 0x01, 0x01);
key_const!(CLOCK_FILTER, 0x01, 0x02);
key_const!(OVERREAD, 0x01, 0x03);
key_const!(DPCM, 0x01, 0x04);
key_const!(GAME_GENIE_CODE, 0x01, 0x05);
key_const!(CONTROLLER, 0x01, 0xF0);


key_const!(INPUT_CHUNKS, 0xFE, 0x01);
key_const!(TRANSITION, 0xFE, 0x02);
key_const!(LAG_FRAME, 0xFE, 0x03);

fn spec_description(key: &[u8]) -> &str {
    match key {
        CONSOLE_TYPE => "ConsoleType - ",
        _ => ""
    }
}

static CONSOLE_TYPE_MAP: phf::Map<u8, &'static str> = phf_map! {
    0x01u8 => "NES",
    0x02u8 => "SNES",
    0x03u8 => "N64",
    0x04u8 => "GC",
    0x05u8 => "GB",
    0x06u8 => "GBC",
    0x07u8 => "GBA",
    0x08u8 => "Genesis",
    0x09u8 => "A2600",
};

static CONSOLE_REGION_MAP: phf::Map<u8, &'static str> = phf_map! {
    0x01u8 => "NTSC",
    0x02u8 => "PAL",
};


#[derive(PartialEq, Clone, Debug, Default)]
pub struct PayloadSize {
    pub exponent: u8,
    pub length_bytes: Vec<u8>,
    pub len: usize,
}
impl Display for PayloadSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02X}, {:02X}", self.exponent, self.len)
    }
}

#[derive(PartialEq, Clone, Debug, Default)]
pub struct PacketRaw {
    key: Vec<u8>,
    size: PayloadSize,
    payload: Vec<u8>,
}
impl Display for PacketRaw {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut payload_str = String::new();
        for byte in &self.payload {
            payload_str.push_str(format!("{:02X} ", byte).as_str());
        }
        
        write!(f, "({:04X}, {}, {})", to_usize(self.key.as_slice()), self.size, payload_str.trim_end())
    }
}
impl PacketRaw {
    pub fn new(key: Vec<u8>, size: PayloadSize, payload: Vec<u8>) -> Self {
        Self {
            key,
            size,
            payload,
        }
    }
}

#[derive(PartialEq, Clone, Debug, EnumIter)]
pub enum Packet {
    // Console-agnostic Keys //
    
    ConsoleType(PacketRaw, u8),
    ConsoleRegion(PacketRaw, u8),
    GameTitle(PacketRaw, String),
    //HashX(),
    Author(PacketRaw, String),
    Category(PacketRaw, String),
    EmulatorName(PacketRaw, String),
    EmulatorVersion(PacketRaw, String),
    
    TasLastModified(PacketRaw, i64),
    DumpLastModified(PacketRaw, i64),
    TotalFrames(PacketRaw, u32),
    Rerecords(PacketRaw, u32),
    SourceLink(PacketRaw, String),
    
    BlankFrames(PacketRaw, i16),
    Verified(PacketRaw, u8),
    
    MemoryInit {
        raw: PacketRaw,
        kind: u8,
        v: Option<u8>,
        k: Option<Vec<u8>>,
        n: Option<String>,
        p: Option<Vec<u8>>,
    },
    
    // NES Keys //
    LatchFilter(PacketRaw, u8),
    ClockFilter(PacketRaw, u8),
    Overread(PacketRaw, u8),
    Dpcm(PacketRaw, u8),
    GameGenieCode(PacketRaw, Vec<u8>),
    
    //NesController(PacketRaw, u8, u8),
    
    
    // SNES Keys //
    //SnesController(PacketRaw, u8, u8),
    
    // N64 Keys //
    
    
    // GB/C/A Keys //
    
    
    // Genesis Keys //
    
    
    // A2600 Keys //
    
    
    // Input Frame/Timing Keys //

    InputChunks(PacketRaw, u8, Vec<u8>),
    Transition(PacketRaw, u32, u8, Vec<u8>),
    LagFrame(PacketRaw, u32, u32),
    
    Unsupported(PacketRaw),
}
impl Display for Packet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Packet::*;
        match self {
            ConsoleType(raw, val) => write!(f, "ConsoleType({}, {})", raw, CONSOLE_TYPE_MAP.get(val).unwrap_or(&"Unknown")),
            ConsoleRegion(raw, val) => write!(f, "ConsoleRegion({}, {})", raw, CONSOLE_REGION_MAP.get(val).unwrap_or(&"Unknown")),
            GameTitle(raw, val) => write!(f, "GameTitle({}, {})", raw, val),
            Author(raw, val) => write!(f, "Author({}, {})", raw, val),
            Category(raw, val) => write!(f, "Category({}, {})", raw, val),
            EmulatorName(raw, val) => write!(f, "EmulatorName({}, {})", raw, val),
            EmulatorVersion(raw, val) => write!(f, "EmulatorVersion({}, {})", raw, val),
            TasLastModified(raw, val) => write!(f, "TasLastModified({}, {})", raw, Utc.timestamp(*val, 0).to_string()),
            DumpLastModified(raw, val) => write!(f, "DumpLastModified({}, {})", raw, Utc.timestamp(*val, 0).to_string()),
            TotalFrames(raw, val) => write!(f, "TotalFrames({}, {})", raw, val),
            Rerecords(raw, val) => write!(f, "Rerecords({}, {})", raw, val),
            SourceLink(raw, val) => write!(f, "SourceLink({}, {})", raw, val),
            MemoryInit { raw, kind, v, k, n, p } => {
                match kind {
                    2 => {
                        let mut payload_str = String::new();
                        for byte in p.clone().unwrap() {
                            payload_str.push_str(format!("{:02X} ", byte).as_str());
                        }
                        
                        write!(f, "MemoryInit({}, {:02X}, {}, [{}])", raw, kind, n.clone().unwrap(), payload_str.trim_end())
                    },
                    1 | 3..=6 => {
                        write!(f, "MemoryInit({}, {:02X}, {})", raw, kind, n.clone().unwrap_or(String::new()))
                    },
                    _ => write!(f, "MemoryInit({}, kind={:02X}, v={:?}, k={:?}, n={:?}, p={:?})", raw, kind, v, k, n, p)
                }
                
            },
            BlankFrames(raw, val) => write!(f, "BlankFrames({}, {})", raw, val),
            Verified(raw, val) => write!(f, "Verified({}, {})", raw, val),
            _ => write!(f, "{:?}", self)
        }
    }
}
impl From<Packet> for String {
    fn from(packet: Packet) -> Self {
        format!("{}", packet)
    }
}
impl From<&Packet> for String {
    fn from(packet: &Packet) -> Self {
        format!("{}", packet)
    }
}
impl Packet {
    pub fn parse(key_width: usize, data: &Vec<u8>, i: &mut usize) -> Result<Self, (Self, String)> {
        use Packet::*;
        
        fn payload_error(raw: PacketRaw, message: &str) -> Result<Packet, (Packet, String)> { Err((Unsupported(raw), String::from(message))) }
        
        let key = &data[*i..(*i + key_width)];
        *i += key_width;
        let size = payload_len(&data, i);
        let payload = &data[*i..(*i + size.len)];
        *i += size.len;
        
        let raw = PacketRaw::new(key.to_vec(), size.clone(), payload.to_vec());
        let rawc = raw.clone();
        let packet: Packet;
        loop { match key {
            /* --- Console-agnostic Keys --- */
            
            CONSOLE_TYPE => {
                if size.len != 1 { return payload_error(raw, "Payload length is invalid for CONSOLE_TYPE"); }
                packet = ConsoleType(rawc, raw.payload[0]);
            },
            CONSOLE_REGION => {
                if size.len != 1 { return payload_error(raw, "Payload length is invalid for CONSOLE_REGION"); }
                packet = ConsoleRegion(rawc, raw.payload[0]);
            },
            GAME_TITLE => { packet = GameTitle(rawc, String::from_utf8_lossy(raw.payload.as_slice()).into()); },
            //TODO hashes
            
            AUTHOR => { packet = Author(rawc, String::from_utf8_lossy(raw.payload.as_slice()).into()); },
            CATEGORY => { packet = Category(rawc, String::from_utf8_lossy(raw.payload.as_slice()).into()); },
            EMULATOR_NAME => { packet = EmulatorName(rawc, String::from_utf8_lossy(raw.payload.as_slice()).into()); },
            EMULATOR_VERSION => { packet = EmulatorVersion(rawc, String::from_utf8_lossy(raw.payload.as_slice()).into()); },
            
            TAS_LAST_MODIFIED => {
                if size.len != 8 { return payload_error(raw, "Payload length is invalid for TAS_LAST_MODIFIED"); }
                packet = TasLastModified(rawc, to_u64(&raw.payload) as i64);
            },
            DUMP_LAST_MODIFIED => {
                if size.len != 8 { return payload_error(raw, "Payload length is invalid for DUMP_LAST_MODIFIED"); }
                packet = DumpLastModified(rawc, to_u64(&raw.payload) as i64);
            },
            TOTAL_FRAMES => {
                if size.len != 4 { return payload_error(raw, "Payload length is invalid for TOTAL_FRAMES"); }
                packet = TotalFrames(rawc, to_u32(&raw.payload));
            },
            RERECORDS => {
                if size.len != 4 { return payload_error(raw, "Payload length is invalid for RERECORDS"); }
                packet = Rerecords(rawc, to_u32(&raw.payload));
            },
            SOURCE_LINK => { packet = SourceLink(rawc, String::from_utf8_lossy(raw.payload.as_slice()).into()); },
            
            MEMORY_INIT => {
                if size.len == 0 { return payload_error(raw, "Payload length is invalid for MEMORY_INIT"); }
                let kind = raw.payload[0];
                let v = raw.payload[1];
                let k = raw.payload[2..(2 + v as usize)].to_vec();
                let size = payload_len(&raw.payload, &mut 1);
                let n: String = String::from_utf8_lossy(&raw.payload[3..(3 + size.len)]).into();
                
                if kind == 2 {
                    packet = MemoryInit { raw: rawc, kind,  v: Some(v), k: Some(k), n: Some(n), p: Some(raw.payload[(3 + 2 + 1 + v as usize)..raw.payload.len()].to_vec()) };
                } else {
                    packet = MemoryInit { raw, kind, v: Some(v), k: Some(k), n: Some(n), p: None };
                }
            }
            
            BLANK_FRAMES => {
                if size.len != 2 { return payload_error(raw, "Payload length is invalid for BLANK_FRAMES"); }
                packet = BlankFrames(rawc, to_i16(&raw.payload));
            },
            VERIFIED => {
                if size.len != 1 { return payload_error(raw, "Payload length is invalid for VERIFIED"); }
                packet = Verified(rawc, raw.payload[0]);
            },
            
            /* --- NES Keys --- */
            
            LATCH_FILTER => {
                if size.len != 1 { return payload_error(raw, "Payload length is invalid for LATCH_FILTER"); }
                packet = LatchFilter(rawc, raw.payload[0]);
            },
            CLOCK_FILTER => {
                if size.len != 1 { return payload_error(raw, "Payload length is invalid for CLOCK_FILTER"); }
                packet = ClockFilter(rawc, raw.payload[0]);
            },
            OVERREAD => {
                if size.len != 1 { return payload_error(raw, "Payload length is invalid for OVERREAD"); }
                packet = Overread(rawc, raw.payload[0]);
            },
            DPCM => {
                if size.len != 1 { return payload_error(raw, "Payload length is invalid for DPCM"); }
                packet = Dpcm(rawc, raw.payload[0]);
            },
            GAME_GENIE_CODE => {
                if size.len != 6 && size.len != 8 { return payload_error(raw, "Payload length is invalid for GAME_GENIE_CODE"); }
                packet = GameGenieCode(rawc, raw.payload);
            },
            
            /* --- Input Frame/Timing Keys --- */
            
            INPUT_CHUNKS => {
                if size.len < 2 { return payload_error(raw, "Payload length is invalid for INPUT_CHUNKS"); }
                packet = InputChunks(rawc, raw.payload[0], raw.payload[1..raw.payload.len()].to_vec());
            },
            TRANSITION => {
                if size.len < 5 { return payload_error(raw, "Payload length is invalid for TRANSITION"); }
                packet = Transition(rawc, to_u32(&raw.payload[0..4]), raw.payload[4], raw.payload[5..raw.payload.len()].to_vec());
            },
            LAG_FRAME => {
                if size.len != 8 { return payload_error(raw, "Payload length is invalid for LAG_FRAME"); }
                packet = LagFrame(rawc, to_u32(&raw.payload[0..4]), to_u32(&raw.payload[4..8]));
            }
            
            
            _ => packet = Unsupported(raw)
        } break; }
        
        Ok(packet)
    }
}

#[derive(Default, Clone, Debug)]
pub struct TasdMovie {
    pub version: u16,
    pub key_width: u8,
    pub packets: Vec<Packet>,
}
impl TasdMovie {
    pub fn new(data: &Vec<u8>) -> Result<Self, String> {
        if &data[0..4] != MAGIC_NUMBER {
            return Err(String::from("Magic Number doesn't match. This file doesn't appear to be a TASD."));
        }
        
        let mut tasd = Self::default();
        tasd.version = to_u16(&data[4..=5]);
        tasd.key_width = data[6];
        
        let mut i = 7;
        loop {
            if i >= data.len() { break; }
            
            let result = Packet::parse(tasd.key_width as usize, data, &mut i);
            if result.is_ok() {
                tasd.packets.push(result.unwrap());
            } else {
                let error = result.err().unwrap();
                tasd.packets.push(error.0);
                println!("[Warning] {}", error.1);
            }
        }
        
        Ok(tasd)
    }
}



fn to_i16(mut data: &[u8]) -> i16 {
    data.read_i16::<BigEndian>().unwrap()
}

fn to_u16(mut data: &[u8]) -> u16 {
    data.read_u16::<BigEndian>().unwrap()
}

fn to_u32(mut data: &[u8]) -> u32 {
    data.read_u32::<BigEndian>().unwrap()
}

fn to_u64(mut data: &[u8]) -> u64 {
    data.read_u64::<BigEndian>().unwrap()
}

fn to_usize(data: &[u8]) -> usize {
    let mut out = 0;
    
    for by in data {
        out = (out << 8) | (*by as usize);
    }
    
    out
}

fn payload_len(data: &Vec<u8>, i: &mut usize) -> PayloadSize {
    let len_spec = data[*i] as usize;
    *i += 1;
    
    let len_bytes = &data[*i..(*i + len_spec)];
    let len = to_usize(len_bytes);
    *i += len_spec;
    
    PayloadSize {
        exponent: len_spec as u8,
        length_bytes: len_bytes.to_vec(),
        len: len
    }
}




