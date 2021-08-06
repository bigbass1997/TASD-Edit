use std::convert::TryInto;
use byteorder::{ReadBytesExt, BigEndian};
use std::fmt::{Display, Formatter};
use crate::spec::Packet::{ConsoleType, Unsupported};

macro_rules! key_const {
    ($name:ident, $upper:expr, $lower:expr) => {
        pub const $name: &[u8] = &[$upper, $lower];
    };
}


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
key_const!(MEMORY_INIT, 0x00, 0x0E);
key_const!(BLANK_FRAMES, 0x00, 0x0F);
key_const!(VERIFIED, 0x00, 0x10);


#[derive(PartialEq, Clone, Debug)]
pub struct PayloadSize {
    pub exponent: u8,
    pub length_bytes: Vec<u8>,
    pub len: usize,
}

#[derive(PartialEq, Clone, Debug)]
pub struct PacketRaw {
    key: Vec<u8>,
    size: PayloadSize,
    payload: Vec<u8>,
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

#[derive(PartialEq, Clone, Debug)]
pub enum Packet {
    // Console-agnostic Keys //
    
    ConsoleType(PacketRaw, u8),
    ConsoleRegion(u8),
    GameTitle(String),
    //HashX(),
    Author(String),
    Category(String),
    EmulatorName(String),
    EmulatorVersion(String),
    
    TasLastModified(u64),
    DumpLastModified(u64),
    TotalFrames(u32),
    Rerecords(u32),
    SourceLink(String),
    
    MemoryInit {
        kind: u8,
        v: Option<u8>,
        k: Option<Vec<u8>>,
        n: Option<String>,
        p: Option<Vec<u8>>,
    },
    
    BlankFrames(i16),
    Verified(u8),
    
    // NES Keys //
    
    
    // SNES Keys //
    
    
    // N64 Keys //
    
    
    // GB/C/A Keys //
    
    
    // Genesis Keys //
    
    
    // A2600 Keys //
    
    
    // Input Frame/Timing Keys //
    
    Input(Vec<u8>),
    TransitionFrame((u32, u8, Option<Vec<u8>>)),
    LagFrame((u32, u32)),
    
    Unsupported(PacketRaw),
}
impl Packet {
    pub fn parse(key_width: usize, data: &Vec<u8>, i: &mut usize) -> Result<Self, (Self, String)> {
        
        fn payload_error(raw: PacketRaw, message: &str) -> Result<Packet, (Packet, String)> { Err((Unsupported(raw), String::from(message))) }
        
        let key = &data[*i..(*i + key_width)];
        *i += key_width;
        let size = payload_len(&data, i);
        let payload = &data[*i..(*i + size.len)];
        *i += size.len;
        
        let raw = PacketRaw::new(key.to_vec(), size, payload.to_vec());
        let mut packet: Packet;
        loop { match key {
            CONSOLE_TYPE => {
                if size.len == 0 { return payload_error(raw, "Payload length is zero"); }
                if size.len > 1 { return payload_error(raw, "Payload length too long for CONSOLE_TYPE"); }
                packet = Packet::ConsoleType(raw, raw.payload[0]);
            },
            
            _ => packet = Unsupported(raw)
        } break; }
        
        Ok(packet)
    }
}
/*impl Display for Packet {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ConsoleType(val) => write!(f, "(ConsoleType, "),
            
            _ => write!(f, "{:?}", self)
        }
    }
}*/

#[derive(Default, Clone, Debug)]
pub struct TasdMovie {
    pub version: u16,
    pub key_width: u8,
    pub packets: Vec<Packet>,
}
impl TasdMovie {
    pub fn new(data: &Vec<u8>) -> Result<Self, String> {
        if data[0..4] != [0x54, 0x41, 0x53, 0x44] {
            return Err(String::from("Magic Number doesn't match."));
        }
        
        fn payload_len_err_u16(len: usize, key: &[u8]) -> Result<TasdMovie, String> { Err(format!("Payload length of {}, unsupported for key {:04X}", len, to_u16(key))) }
        
        let mut tasd = Self::default();
        tasd.version = to_u16(&data[4..=5]);
        tasd.key_width = data[6];
        
        let kw = tasd.key_width as usize;
        let mut i = 7;
        for _ in i..data.len() {
            if i >= data.len() { break; }
            
            let key = &data[i..(i + kw)];
            i += kw;
            println!("Key: {:02X} {:02X}", key[0], key[1]);
            
            loop { match key {
                CONSOLE_TYPE => {
                    let len = payload_len(&data, &mut i);
                    if len == 0 { break; }
                    if len > 1 { return payload_len_err_u16(len, key); }
                    tasd.packets.push(Packet::ConsoleType(data[i]));
                    i += 1;
                },
                CONSOLE_REGION => {
                    let len = payload_len(&data, &mut i);
                    if len == 0 { break; }
                    if len > 1 { return payload_len_err_u16(len, key); }
                    tasd.packets.push(Packet::ConsoleRegion(data[i]));
                    i += 1;
                },
                GAME_TITLE => {
                    let len = payload_len(&data, &mut i);
                    if len == 0 { break; }
                    tasd.packets.push(Packet::GameTitle(String::from_utf8_lossy(&data[i..(i + len)]).into()));
                    i += len;
                },
                //TODO hashes
                
                AUTHOR => {
                    let len = payload_len(&data, &mut i);
                    if len == 0 { break; }
                    tasd.packets.push(Packet::Author(String::from_utf8_lossy(&data[i..(i + len)]).into()));
                    i += len;
                },
                CATEGORY => {
                    let len = payload_len(&data, &mut i);
                    if len == 0 { break; }
                    tasd.packets.push(Packet::Category(String::from_utf8_lossy(&data[i..(i + len)]).into()));
                    i += len;
                },
                EMULATOR_NAME => {
                    let len = payload_len(&data, &mut i);
                    if len == 0 { break; }
                    tasd.packets.push(Packet::EmulatorName(String::from_utf8_lossy(&data[i..(i + len)]).into()));
                    i += len;
                },
                EMULATOR_VERSION => {
                    let len = payload_len(&data, &mut i);
                    if len == 0 { break; }
                    tasd.packets.push(Packet::EmulatorVersion(String::from_utf8_lossy(&data[i..(i + len)]).into()));
                    i += len;
                },
                
                TAS_LAST_MODIFIED => {
                    let len = payload_len(&data, &mut i);
                    if len == 0 { break; }
                    if len != 8 { return payload_len_err_u16(len, key); }
                    tasd.packets.push(Packet::TasLastModified(to_u64(&data[i..(i + len)])));
                    i += len;
                },
                DUMP_LAST_MODIFIED => {
                    let len = payload_len(&data, &mut i);
                    if len == 0 { break; }
                    if len != 8 { return payload_len_err_u16(len, key); }
                    tasd.packets.push(Packet::DumpLastModified(to_u64(&data[i..(i + len)])));
                    i += len;
                },
                TOTAL_FRAMES => {
                    let len = payload_len(&data, &mut i);
                    if len == 0 { break; }
                    if len != 4 { return payload_len_err_u16(len, key); }
                    tasd.packets.push(Packet::TotalFrames(to_u32(&data[i..(i + len)])));
                    i += len;
                },
                RERECORDS => {
                    let len = payload_len(&data, &mut i);
                    if len == 0 { break; }
                    if len != 4 { return payload_len_err_u16(len, key); }
                    tasd.packets.push(Packet::Rerecords(to_u32(&data[i..(i + len)])));
                    i += len;
                },
                SOURCE_LINK => {
                    let len = payload_len(&data, &mut i);
                    if len == 0 { break; }
                    tasd.packets.push(Packet::SourceLink(String::from_utf8_lossy(&data[i..(i + len)]).into()));
                    i += len;
                },
                
                MEMORY_INIT => {
                    let len = payload_len(&data, &mut i);
                    if len == 0 { break; }
                    let kind = data[i];
                    i += 1;
                    if kind == 1 {
                        tasd.packets.push(Packet::MemoryInit { kind, v: None, k: None, n: None, p: None });
                        break;
                    }
                    
                    let v = data[i];
                    let k = data[(i + 1)..(i + 1 + v as usize)].to_vec();
                    let len = payload_len(&data, &mut i);
                    let n: String = String::from_utf8_lossy(&data[i..(i + len)]).into();
                    
                    match kind {
                        2 => {
                            
                        },
                        3..=6 => tasd.packets.push(Packet::MemoryInit { kind, v: Some(v), k: Some(k), n: Some(n), p: None }),
                        
                        _ => ()
                    }
                },
                
                BLANK_FRAMES => {
                    let len = payload_len(&data, &mut i);
                    if len == 0 { break; }
                    if len != 2 { return payload_len_err_u16(len, key); }
                    tasd.packets.push(Packet::BlankFrames(to_i16(&data[i..(i + len)])));
                    i += len;
                },
                VERIFIED => {
                    let len = payload_len(&data, &mut i);
                    if len == 0 { break; }
                    if len > 1 { return payload_len_err_u16(len, key); }
                    tasd.packets.push(Packet::Verified(data[i]));
                    i += 1;
                },
                
                
                _ => i += payload_len(&data, &mut i)
            } break; }
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
    
    /*match len_spec {
        1 => data[*i] as usize,
        2 => to_u16(&data[*i..(*i + 2)]) as usize,
        4 => to_u32(&data[*i..(*i + 4)]) as usize,
        8 => to_u64(&data[*i..(*i + 8)]) as usize,
        _ => 0
    }*/
}




