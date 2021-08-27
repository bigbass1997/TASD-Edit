
use crate::lookup::{console_type_lut, console_region_lut, memory_init_lut, transition_lut, controller_type_lut};
use std::fmt::{Debug, Display};
use dyn_clone::DynClone;
use crate::util::{to_bytes, to_i64, to_u32, to_i16, to_usize, to_u16};
use chrono::{Utc, TimeZone};
use crate::movie::parse_packet;

macro_rules! key_const {
    ($name:ident, $upper:expr, $lower:expr) => {
        pub const $name: [u8; 2] = [$upper, $lower];
    };
}

pub const MAGIC_NUMBER: [u8; 4] = [0x54, 0x41, 0x53, 0x44];
pub const NEW_TASD_FILE: [u8; 7] = [0x54, 0x41, 0x53, 0x44, 0x00, 0x01, 0x02];

key_const!(CONSOLE_TYPE, 0x00, 0x01);
key_const!(CONSOLE_REGION, 0x00, 0x02);
key_const!(GAME_TITLE, 0x00, 0x03);
key_const!(AUTHOR, 0x00, 0x04);
key_const!(CATEGORY, 0x00, 0x05);
key_const!(EMULATOR_NAME, 0x00, 0x06);
key_const!(EMULATOR_VERSION, 0x00, 0x07);
key_const!(EMULATOR_CORE, 0x00, 0x08);
key_const!(TAS_LAST_MODIFIED, 0x00, 0x09);
key_const!(DUMP_LAST_MODIFIED, 0x00, 0x0A);
key_const!(TOTAL_FRAMES, 0x00, 0x0B);
key_const!(RERECORDS, 0x00, 0x0C);
key_const!(SOURCE_LINK, 0x00, 0x0D);
key_const!(BLANK_FRAMES, 0x00, 0x0E);
key_const!(VERIFIED, 0x00, 0x0F);
key_const!(MEMORY_INIT, 0x00, 0x10);

key_const!(PORT_CONTROLLER, 0x00, 0xF0);

key_const!(LATCH_FILTER, 0x01, 0x01);
key_const!(CLOCK_FILTER, 0x01, 0x02);
key_const!(OVERREAD, 0x01, 0x03);
key_const!(GAME_GENIE_CODE, 0x01, 0x04);



key_const!(INPUT_CHUNKS, 0xFE, 0x01);
key_const!(TRANSITION, 0xFE, 0x02);
key_const!(LAG_FRAME_CHUNK, 0xFE, 0x03);
key_const!(MOVIE_TRANSITION, 0xFE, 0x04);


#[derive(Clone, Debug, Default)]
pub struct PacketSpec {
    pub key: [u8; 2],
    pub name: String,
    pub description: String,
}
impl Display for PacketSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {}", self.name, self.description)
    }
}

macro_rules! packet_spec {
    ($key:expr, $name:expr, $description:expr) => {
        PacketSpec { key: $key, name: String::from($name), description: String::from($description) }
    };
}




pub trait Packet: DynClone {
    fn parse(data: &Vec<u8>, i: &mut usize) -> Box<dyn Packet> where Self: Sized;
    
    fn get_raw(&self) -> Vec<u8>;
    
    fn get_packet_spec(&self) -> PacketSpec;
    fn formatted_payload(&self) -> String;
}
impl Debug for dyn Packet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
dyn_clone::clone_trait_object!(Packet);



////////////////////////////////////// Unsupported //////////////////////////////////////
#[derive(Default, Clone, Debug)]
pub struct Unsupported {
    raw: Vec<u8>,
}
impl Packet for Unsupported {
    fn parse(data: &Vec<u8>, i: &mut usize) -> Box<dyn Packet> {
        let (raw, payload) = get_raw_packet(data, i);
        Box::new(Self { raw })
    }
    
    fn get_raw(&self) -> Vec<u8> {
        self.raw.clone()
    }
    
    fn get_packet_spec(&self) -> PacketSpec {
        packet_spec!([0x00, 0x00], "Unsupported", "Packet is not known to this software.")
    }
    fn formatted_payload(&self) -> String {
        let mut out = String::new();
        for byte in &self.raw {
            out.push_str(format!("{:02X} ", byte).as_str());
        }
        
        out.trim_end().to_string()
    }
}


////////////////////////////////////// ConsoleType //////////////////////////////////////
#[derive(Default, Clone, Debug)]
pub struct ConsoleType {
    raw: Vec<u8>,
    kind: u8,
    custom: Option<String>,
}
impl ConsoleType {
    pub fn new(kind: u8, custom: Option<String>) -> Box<Self> {
        Box::new(Self {
            raw: serialize_payload(CONSOLE_TYPE.to_vec(), vec![kind]),
            kind: kind,
            custom: custom,
        })
    }
}
impl Packet for ConsoleType {
    fn parse(data: &Vec<u8>, i: &mut usize) -> Box<dyn Packet> {
        let (raw, payload) = get_raw_packet(data, i);
        let mut custom = None;
        if payload[0] == 0xFF {
            custom = Some(String::from_utf8_lossy(&payload[1..payload.len()]).to_string());
        }
        
        Box::new(Self {
            raw: raw,
            kind: payload[0],
            custom: custom,
        })
    }
    
    fn get_raw(&self) -> Vec<u8> {
        self.raw.clone()
    }
    
    fn get_packet_spec(&self) -> PacketSpec {
        packet_spec!(CONSOLE_TYPE, "ConsoleType", "The console this TAS is made for.")
    }
    fn formatted_payload(&self) -> String {
        if self.custom.is_some() {
            format!("{}: {}", console_type_lut(self.kind).unwrap_or("Unknown"), self.custom.clone().unwrap())
        } else {
            console_type_lut(self.kind).unwrap_or("Unknown").to_string()
        }
    }
}


////////////////////////////////////// ConsoleRegion //////////////////////////////////////
#[derive(Default, Clone, Debug)]
pub struct ConsoleRegion {
    raw: Vec<u8>,
    payload: u8,
}
impl ConsoleRegion {
    pub fn new(kind: u8) -> Box<Self> {
        Box::new(Self {
            raw: serialize_payload(CONSOLE_REGION.to_vec(), vec![kind]),
            payload: kind,
        })
    }
}
impl Packet for ConsoleRegion {
    fn parse(data: &Vec<u8>, i: &mut usize) -> Box<dyn Packet> {
        let (raw, payload) = get_raw_packet(data, i);
        
        Box::new(Self {
            raw: raw,
            payload: payload[0],
        })
    }
    
    fn get_raw(&self) -> Vec<u8> {
        self.raw.clone()
    }
    
    fn get_packet_spec(&self) -> PacketSpec {
        packet_spec!(CONSOLE_REGION, "ConsoleRegion", "Console region required to play this TAS.")
    }
    fn formatted_payload(&self) -> String {
        console_region_lut(self.payload).unwrap_or("Unknown").to_string()
    }
}


////////////////////////////////////// GameTitle //////////////////////////////////////
#[derive(Default, Clone, Debug)]
pub struct GameTitle {
    raw: Vec<u8>,
    payload: String,
}
impl GameTitle {
    pub fn new(title: String) -> Box<Self> {
        Box::new(Self {
            raw: serialize_payload(GAME_TITLE.to_vec(), title.as_bytes().to_vec()),
            payload: title,
        })
    }
}
impl Packet for GameTitle {
    fn parse(data: &Vec<u8>, i: &mut usize) -> Box<dyn Packet> {
        let (raw, payload) = get_raw_packet(data, i);
        
        Box::new(Self {
            raw: raw,
            payload: String::from_utf8_lossy(payload.as_slice()).to_string(),
        })
    }
    
    fn get_raw(&self) -> Vec<u8> {
        self.raw.clone()
    }
    
    fn get_packet_spec(&self) -> PacketSpec {
        packet_spec!(GAME_TITLE, "GameTitle", "(string) Title of the game.")
    }
    fn formatted_payload(&self) -> String {
        self.payload.clone()
    }
}


////////////////////////////////////// Author //////////////////////////////////////
#[derive(Default, Clone, Debug)]
pub struct Author {
    raw: Vec<u8>,
    payload: String,
}
impl Author {
    pub fn new(author: String) -> Box<Self> {
        Box::new(Self {
            raw: serialize_payload(AUTHOR.to_vec(), author.as_bytes().to_vec()),
            payload: author,
        })
    }
}
impl Packet for Author {
    fn parse(data: &Vec<u8>, i: &mut usize) -> Box<dyn Packet> {
        let (raw, payload) = get_raw_packet(data, i);
        
        Box::new(Self {
            raw: raw,
            payload: String::from_utf8_lossy(payload.as_slice()).to_string(),
        })
    }
    
    fn get_raw(&self) -> Vec<u8> {
        self.raw.clone()
    }
    
    fn get_packet_spec(&self) -> PacketSpec {
        packet_spec!(AUTHOR, "Author", "(string) Name of one author of the TAS. (e.g. \"Bender B. Rodriguez\")")
    }
    fn formatted_payload(&self) -> String {
        self.payload.clone()
    }
}


////////////////////////////////////// Category //////////////////////////////////////
#[derive(Default, Clone, Debug)]
pub struct Category {
    raw: Vec<u8>,
    payload: String,
}
impl Category {
    pub fn new(category: String) -> Box<Self> {
        Box::new(Self {
            raw: serialize_payload(CATEGORY.to_vec(), category.as_bytes().to_vec()),
            payload: category,
        })
    }
}
impl Packet for Category {
    fn parse(data: &Vec<u8>, i: &mut usize) -> Box<dyn Packet> {
        let (raw, payload) = get_raw_packet(data, i);
        
        Box::new(Self {
            raw: raw,
            payload: String::from_utf8_lossy(payload.as_slice()).to_string(),
        })
    }
    
    fn get_raw(&self) -> Vec<u8> {
        self.raw.clone()
    }
    
    fn get_packet_spec(&self) -> PacketSpec {
        packet_spec!(CATEGORY, "Category", "(string) Category of the TAS. (e.g. \"any%\")")
    }
    fn formatted_payload(&self) -> String {
        self.payload.clone()
    }
}


////////////////////////////////////// EmulatorName //////////////////////////////////////
#[derive(Default, Clone, Debug)]
pub struct EmulatorName {
    raw: Vec<u8>,
    payload: String,
}
impl EmulatorName {
    pub fn new(emulator_name: String) -> Box<Self> {
        Box::new(Self {
            raw: serialize_payload(EMULATOR_NAME.to_vec(), emulator_name.as_bytes().to_vec()),
            payload: emulator_name,
        })
    }
}
impl Packet for EmulatorName {
    fn parse(data: &Vec<u8>, i: &mut usize) -> Box<dyn Packet> {
        let (raw, payload) = get_raw_packet(data, i);
        
        Box::new(Self {
            raw: raw,
            payload: String::from_utf8_lossy(payload.as_slice()).to_string(),
        })
    }
    
    fn get_raw(&self) -> Vec<u8> {
        self.raw.clone()
    }
    
    fn get_packet_spec(&self) -> PacketSpec {
        packet_spec!(EMULATOR_NAME, "EmulatorName", "(string) Name of the emulator used to dump this file.")
    }
    fn formatted_payload(&self) -> String {
        self.payload.clone()
    }
}


////////////////////////////////////// EmulatorVersion //////////////////////////////////////
#[derive(Default, Clone, Debug)]
pub struct EmulatorVersion {
    raw: Vec<u8>,
    payload: String,
}
impl EmulatorVersion {
    pub fn new(emulator_version: String) -> Box<Self> {
        Box::new(Self {
            raw: serialize_payload(EMULATOR_VERSION.to_vec(), emulator_version.as_bytes().to_vec()),
            payload: emulator_version,
        })
    }
}
impl Packet for EmulatorVersion {
    fn parse(data: &Vec<u8>, i: &mut usize) -> Box<dyn Packet> {
        let (raw, payload) = get_raw_packet(data, i);
        
        Box::new(Self {
            raw: raw,
            payload: String::from_utf8_lossy(payload.as_slice()).to_string(),
        })
    }
    
    fn get_raw(&self) -> Vec<u8> {
        self.raw.clone()
    }
    
    fn get_packet_spec(&self) -> PacketSpec {
        packet_spec!(EMULATOR_VERSION, "EmulatorVersion", "(string) Version of the emulator.")
    }
    fn formatted_payload(&self) -> String {
        self.payload.clone()
    }
}


////////////////////////////////////// EmulatorCore //////////////////////////////////////
#[derive(Default, Clone, Debug)]
pub struct EmulatorCore {
    raw: Vec<u8>,
    payload: String,
}
impl EmulatorCore {
    pub fn new(emulator_core: String) -> Box<Self> {
        Box::new(Self {
            raw: serialize_payload(EMULATOR_CORE.to_vec(), emulator_core.as_bytes().to_vec()),
            payload: emulator_core,
        })
    }
}
impl Packet for EmulatorCore {
    fn parse(data: &Vec<u8>, i: &mut usize) -> Box<dyn Packet> {
        let (raw, payload) = get_raw_packet(data, i);
        
        Box::new(Self {
            raw: raw,
            payload: String::from_utf8_lossy(payload.as_slice()).to_string(),
        })
    }
    
    fn get_raw(&self) -> Vec<u8> {
        self.raw.clone()
    }
    
    fn get_packet_spec(&self) -> PacketSpec {
        packet_spec!(EMULATOR_CORE, "EmulatorCore", "(string) Name of emulation core being used. (may not be applicable to all emulators)")
    }
    fn formatted_payload(&self) -> String {
        self.payload.clone()
    }
}


////////////////////////////////////// TASLastModified //////////////////////////////////////
#[derive(Default, Clone, Debug)]
pub struct TASLastModified {
    raw: Vec<u8>,
    payload: i64,
}
impl TASLastModified {
    pub fn new(epoch: i64) -> Box<Self> {
        Box::new(Self {
            raw: serialize_payload(TAS_LAST_MODIFIED.to_vec(), to_bytes(epoch as usize, 8)),
            payload: epoch,
        })
    }
}
impl Packet for TASLastModified {
    fn parse(data: &Vec<u8>, i: &mut usize) -> Box<dyn Packet> {
        let (raw, payload) = get_raw_packet(data, i);
        
        Box::new(Self {
            raw: raw,
            payload: to_i64(&payload),
        })
    }
    
    fn get_raw(&self) -> Vec<u8> {
        self.raw.clone()
    }
    
    fn get_packet_spec(&self) -> PacketSpec {
        packet_spec!(TAS_LAST_MODIFIED, "TASLastModified", "(Unix epoch in seconds) Last time the TAS movie was edited. Usually TASVideos.org publication date.")
    }
    fn formatted_payload(&self) -> String {
        Utc.timestamp(self.payload, 0).to_string()
    }
}


////////////////////////////////////// DumpLastModified //////////////////////////////////////
#[derive(Default, Clone, Debug)]
pub struct DumpLastModified {
    raw: Vec<u8>,
    payload: i64,
}
impl DumpLastModified {
    pub fn new(epoch: i64) -> Box<Self> {
        Box::new(Self {
            raw: serialize_payload(DUMP_LAST_MODIFIED.to_vec(), to_bytes(epoch as usize, 8)),
            payload: epoch,
        })
    }
}
impl Packet for DumpLastModified {
    fn parse(data: &Vec<u8>, i: &mut usize) -> Box<dyn Packet> {
        let (raw, payload) = get_raw_packet(data, i);
        
        Box::new(Self {
            raw: raw,
            payload: to_i64(&payload),
        })
    }
    
    fn get_raw(&self) -> Vec<u8> {
        self.raw.clone()
    }
    
    fn get_packet_spec(&self) -> PacketSpec {
        packet_spec!(DUMP_LAST_MODIFIED, "DumpLastModified", "(Unix Epoch in seconds) Last time this file was edited.")
    }
    fn formatted_payload(&self) -> String {
        Utc.timestamp(self.payload, 0).to_string()
    }
}


////////////////////////////////////// TotalFrames //////////////////////////////////////
#[derive(Default, Clone, Debug)]
pub struct TotalFrames {
    raw: Vec<u8>,
    payload: u32,
}
impl TotalFrames {
    pub fn new(frames: u32) -> Box<Self> {
        Box::new(Self {
            raw: serialize_payload(TOTAL_FRAMES.to_vec(), to_bytes(frames as usize, 8)),
            payload: frames,
        })
    }
}
impl Packet for TotalFrames {
    fn parse(data: &Vec<u8>, i: &mut usize) -> Box<dyn Packet> {
        let (raw, payload) = get_raw_packet(data, i);
        
        Box::new(Self {
            raw: raw,
            payload: to_u32(&payload),
        })
    }
    
    fn get_raw(&self) -> Vec<u8> {
        self.raw.clone()
    }
    
    fn get_packet_spec(&self) -> PacketSpec {
        packet_spec!(TOTAL_FRAMES, "TotalFrames", "Total number of frames from original movie, including lag frames. (useful for calculating movie length)")
    }
    fn formatted_payload(&self) -> String {
        format!("{}", self.payload)
    }
}


////////////////////////////////////// Rerecords //////////////////////////////////////
#[derive(Default, Clone, Debug)]
pub struct Rerecords {
    raw: Vec<u8>,
    payload: u32,
}
impl Rerecords {
    pub fn new(rerecords: u32) -> Box<Self> {
        Box::new(Self {
            raw: serialize_payload(RERECORDS.to_vec(), to_bytes(rerecords as usize, 8)),
            payload: rerecords,
        })
    }
}
impl Packet for Rerecords {
    fn parse(data: &Vec<u8>, i: &mut usize) -> Box<dyn Packet> {
        let (raw, payload) = get_raw_packet(data, i);
        
        Box::new(Self {
            raw: raw,
            payload: to_u32(&payload),
        })
    }
    
    fn get_raw(&self) -> Vec<u8> {
        self.raw.clone()
    }
    
    fn get_packet_spec(&self) -> PacketSpec {
        packet_spec!(RERECORDS, "Rerecords", "TAS rerecord count.")
    }
    fn formatted_payload(&self) -> String {
        format!("{}", self.payload)
    }
}


////////////////////////////////////// SourceLink //////////////////////////////////////
#[derive(Default, Clone, Debug)]
pub struct SourceLink {
    raw: Vec<u8>,
    payload: String,
}
impl SourceLink {
    pub fn new(link: String) -> Box<Self> {
        Box::new(Self {
            raw: serialize_payload(SOURCE_LINK.to_vec(), link.as_bytes().to_vec()),
            payload: link,
        })
    }
}
impl Packet for SourceLink {
    fn parse(data: &Vec<u8>, i: &mut usize) -> Box<dyn Packet> {
        let (raw, payload) = get_raw_packet(data, i);
        
        Box::new(Self {
            raw: raw,
            payload: String::from_utf8_lossy(payload.as_slice()).to_string(),
        })
    }
    
    fn get_raw(&self) -> Vec<u8> {
        self.raw.clone()
    }
    
    fn get_packet_spec(&self) -> PacketSpec {
        packet_spec!(SOURCE_LINK, "SourceLink", "(string) URL link to publication, video upload of this TAS, or any other relevant websites.")
    }
    fn formatted_payload(&self) -> String {
        self.payload.clone()
    }
}


////////////////////////////////////// BlankFrames //////////////////////////////////////
#[derive(Default, Clone, Debug)]
pub struct BlankFrames {
    raw: Vec<u8>,
    payload: i16,
}
impl BlankFrames {
    pub fn new(frames: i16) -> Box<Self> {
        Box::new(Self {
            raw: serialize_payload(BLANK_FRAMES.to_vec(), to_bytes(frames as usize, 8)),
            payload: frames,
        })
    }
}
impl Packet for BlankFrames {
    fn parse(data: &Vec<u8>, i: &mut usize) -> Box<dyn Packet> {
        let (raw, payload) = get_raw_packet(data, i);
        
        Box::new(Self {
            raw: raw,
            payload: to_i16(&payload),
        })
    }
    
    fn get_raw(&self) -> Vec<u8> {
        self.raw.clone()
    }
    
    fn get_packet_spec(&self) -> PacketSpec {
        packet_spec!(BLANK_FRAMES, "BlankFrames", "Signed 16-bit number of blank frames to prepend to the TAS inputs (positive number), or frames to ignore from the start of the TAS (negative number).")
    }
    fn formatted_payload(&self) -> String {
        format!("{}", self.payload)
    }
}


////////////////////////////////////// Verified //////////////////////////////////////
#[derive(Default, Clone, Debug)]
pub struct Verified {
    raw: Vec<u8>,
    payload: u8,
}
impl Verified {
    pub fn new(verified: u8) -> Box<Self> {
        Box::new(Self {
            raw: serialize_payload(VERIFIED.to_vec(), vec![verified]),
            payload: verified,
        })
    }
}
impl Packet for Verified {
    fn parse(data: &Vec<u8>, i: &mut usize) -> Box<dyn Packet> {
        let (raw, payload) = get_raw_packet(data, i);
        
        Box::new(Self {
            raw: raw,
            payload: payload[0],
        })
    }
    
    fn get_raw(&self) -> Vec<u8> {
        self.raw.clone()
    }
    
    fn get_packet_spec(&self) -> PacketSpec {
        packet_spec!(VERIFIED, "Verified", "Whether or not this TAS has been verified by someone. (boolean, value of either 00 or 01)")
    }
    fn formatted_payload(&self) -> String {
        match self.payload {
            0 => "No".to_string(),
            1 => "Yes".to_string(),
            _ => format!("Unknown ({:02X})", self.payload)
        }
    }
}


////////////////////////////////////// MemoryInit //////////////////////////////////////
#[derive(Default, Clone, Debug)]
pub struct MemoryInit {
    raw: Vec<u8>,
    kind: u8,
    required: u8,
    name: String,
    payload: Option<Vec<u8>>,
}
impl MemoryInit {
    pub fn new(kind: u8, required: u8, name: String, payload: Option<Vec<u8>>) -> Box<Self> {
        let mut serialize_data = Vec::new();
        serialize_data.push(kind); // kind
        serialize_data.push(required); // required for verification
        serialize_payload(Vec::new(), name.as_bytes().to_vec()).iter().for_each(|byte| serialize_data.push(*byte)); // v + k + n
        if payload.is_some() {
            payload.clone().unwrap().iter().for_each(|byte| serialize_data.push(*byte)); // p (optional payload)
        }
        
        Box::new(Self {
            raw: serialize_payload(MEMORY_INIT.to_vec(), serialize_data),
            kind: kind,
            required: required,
            name: name,
            payload: payload,
        })
    }
}
impl Packet for MemoryInit {
    fn parse(data: &Vec<u8>, i: &mut usize) -> Box<dyn Packet> {
        let (raw, payload) = get_raw_packet(data, i);
        let kind = payload[0];
        let required = payload[1];
        let name_exp = payload[2];
        let name_len = to_usize(&payload[3..(3 + name_exp as usize)]);
        let name_start = 3 + name_exp as usize;
        let name_end   = name_start + name_len;
        let mut p: Option<Vec<u8>> = None;
        if payload.get(name_end).is_some() {
            p = Some(payload[name_end..payload.len()].to_vec());
        }
        
        Box::new(Self {
            raw: raw,
            kind: kind,
            required: required,
            name: String::from_utf8_lossy(&payload[name_start..name_end]).to_string(),
            payload: p,
        })
    }
    
    fn get_raw(&self) -> Vec<u8> {
        self.raw.clone()
    }
    
    fn get_packet_spec(&self) -> PacketSpec {
        packet_spec!(MEMORY_INIT, "MemoryInit", "Initialization of named memory space. First byte is the kind of initialization.\nSecond byte is whether or not this is required for verifications (0 = optional, 1 = required). Then the name of the space.\nAnd an optional custom payload. (1 byte type, 1 byte verification requirement, v = 1 byte exponent for k, k = length of n, n = name string, p = memory payload)")
    }
    fn formatted_payload(&self) -> String {
        let mut out = memory_init_lut(self.kind).unwrap_or("Unknown Kind").to_string();
        out.push_str(", Required: ");
        
        let other = format!("Unknown ({:02X})", self.required);
        out.push_str(match self.required {
            0 => "No",
            1 => "Yes",
            _ => other.as_str()
        });
        out.push_str(", Space: ");
        out.push_str(self.name.as_str());
        if self.payload.is_some() {
            out.push_str(", Payload: ");
            self.payload.clone().unwrap().iter().for_each(|byte| out.push_str(format!("{:02X} ", byte).as_str()));
        }
        
        out.trim_end().to_string()
    }
}


////////////////////////////////////// PortController //////////////////////////////////////
#[derive(Default, Clone, Debug)]
pub struct PortController {
    raw: Vec<u8>,
    port: u8,
    controller: u16,
}
impl PortController {
    pub fn new(port: u8, controller: u16) -> Box<Self> {
        let mut raw_payload = Vec::new();
        raw_payload.push(port);
        to_bytes(controller as usize, 2).iter().for_each(|byte| raw_payload.push(*byte));
        
        Box::new(Self {
            raw: serialize_payload(PORT_CONTROLLER.to_vec(), raw_payload),
            port: port,
            controller: controller,
        })
    }
}
impl Packet for PortController {
    fn parse(data: &Vec<u8>, i: &mut usize) -> Box<dyn Packet> {
        let (raw, payload) = get_raw_packet(data, i);
        let port = payload[0];
        let controller = to_u16(&payload[1..=2]);
        
        Box::new(Self {
            raw: raw,
            port: port,
            controller: controller,
        })
    }
    
    fn get_raw(&self) -> Vec<u8> {
        self.raw.clone()
    }
    
    fn get_packet_spec(&self) -> PacketSpec {
        packet_spec!(PORT_CONTROLLER, "PortController", "Specify which controller is plugged into a specific port number (1-indexed). (1 byte Port Number, 2 byte Controller Type)")
    }
    fn formatted_payload(&self) -> String {
        format!("Port #{}, Controller Type: {}", self.port, controller_type_lut(self.controller).unwrap_or(&format!("Unknown ({:02X})", self.controller)))
    }
}


////////////////////////////////////// LatchFilter //////////////////////////////////////
#[derive(Default, Clone, Debug)]
pub struct LatchFilter {
    raw: Vec<u8>,
    payload: u8,
}
impl LatchFilter {
    pub fn new(filter: u8) -> Box<Self> {
        Box::new(Self {
            raw: serialize_payload(LATCH_FILTER.to_vec(), vec![filter]),
            payload: filter,
        })
    }
}
impl Packet for LatchFilter {
    fn parse(data: &Vec<u8>, i: &mut usize) -> Box<dyn Packet> {
        let (raw, payload) = get_raw_packet(data, i);
        
        Box::new(Self {
            raw: raw,
            payload: payload[0],
        })
    }
    
    fn get_raw(&self) -> Vec<u8> {
        self.raw.clone()
    }
    
    fn get_packet_spec(&self) -> PacketSpec {
        packet_spec!(LATCH_FILTER, "LatchFilter", "Latch Filter time span. (value multiplied by 0.1ms; inclusive range of 0.0ms to 25.5ms)")
    }
    fn formatted_payload(&self) -> String {
        format!("{:.1}ms", self.payload as f32 * 0.1f32).to_string()
    }
}


////////////////////////////////////// ClockFilter //////////////////////////////////////
#[derive(Default, Clone, Debug)]
pub struct ClockFilter {
    raw: Vec<u8>,
    payload: u8,
}
impl ClockFilter {
    pub fn new(filter: u8) -> Box<Self> {
        Box::new(Self {
            raw: serialize_payload(CLOCK_FILTER.to_vec(), vec![filter]),
            payload: filter,
        })
    }
}
impl Packet for ClockFilter {
    fn parse(data: &Vec<u8>, i: &mut usize) -> Box<dyn Packet> {
        let (raw, payload) = get_raw_packet(data, i);
        
        Box::new(Self {
            raw: raw,
            payload: payload[0],
        })
    }
    
    fn get_raw(&self) -> Vec<u8> {
        self.raw.clone()
    }
    
    fn get_packet_spec(&self) -> PacketSpec {
        packet_spec!(CLOCK_FILTER, "ClockFilter", "Clock Filter time span. (value multiplied by 0.25us; inclusive range of 0.0us to 63.75us)")
    }
    fn formatted_payload(&self) -> String {
        format!("{:.2}us", self.payload as f32 * 0.25f32).to_string()
    }
}


////////////////////////////////////// Overread //////////////////////////////////////
#[derive(Default, Clone, Debug)]
pub struct Overread {
    raw: Vec<u8>,
    payload: u8,
}
impl Overread {
    pub fn new(overread: u8) -> Box<Self> {
        Box::new(Self {
            raw: serialize_payload(OVERREAD.to_vec(), vec![overread]),
            payload: overread,
        })
    }
}
impl Packet for Overread {
    fn parse(data: &Vec<u8>, i: &mut usize) -> Box<dyn Packet> {
        let (raw, payload) = get_raw_packet(data, i);
        
        Box::new(Self {
            raw: raw,
            payload: payload[0],
        })
    }
    
    fn get_raw(&self) -> Vec<u8> {
        self.raw.clone()
    }
    
    fn get_packet_spec(&self) -> PacketSpec {
        packet_spec!(OVERREAD, "Overread", "The data value to use when overread clock pulses occur. (active-low: 0 = HIGH, 1 = LOW)")
    }
    fn formatted_payload(&self) -> String {
        match self.payload {
            0 => "HIGH".to_string(),
            1 => "LOW".to_string(),
            _ => format!("Unknown ({:02X})", self.payload)
        }
    }
}


////////////////////////////////////// GameGenieCode //////////////////////////////////////
#[derive(Default, Clone, Debug)]
pub struct GameGenieCode {
    raw: Vec<u8>,
    payload: String,
}
impl GameGenieCode {
    pub fn new(link: String) -> Box<Self> {
        Box::new(Self {
            raw: serialize_payload(GAME_GENIE_CODE.to_vec(), link.as_bytes().to_vec()),
            payload: link,
        })
    }
}
impl Packet for GameGenieCode {
    fn parse(data: &Vec<u8>, i: &mut usize) -> Box<dyn Packet> {
        let (raw, payload) = get_raw_packet(data, i);
        
        Box::new(Self {
            raw: raw,
            payload: String::from_utf8_lossy(payload.as_slice()).to_string(),
        })
    }
    
    fn get_raw(&self) -> Vec<u8> {
        self.raw.clone()
    }
    
    fn get_packet_spec(&self) -> PacketSpec {
        packet_spec!(GAME_GENIE_CODE, "GameGenieCode", "(string) 6 or 8 character game genie code.")
    }
    fn formatted_payload(&self) -> String {
        self.payload.clone()
    }
}


////////////////////////////////////// InputChunks //////////////////////////////////////
#[derive(Default, Clone, Debug)]
pub struct InputChunks {
    raw: Vec<u8>,
    port: u8,
    payload: Vec<u8>,
}
impl InputChunks {
    pub fn new(port: u8, chunks: Vec<u8>) -> Box<Self> {
        let mut raw_payload = chunks.clone();
        raw_payload.insert(0, port);
        
        Box::new(Self {
            raw: serialize_payload(INPUT_CHUNKS.to_vec(), raw_payload),
            port: port,
            payload: chunks,
        })
    }
}
impl Packet for InputChunks {
    fn parse(data: &Vec<u8>, i: &mut usize) -> Box<dyn Packet> {
        let (raw, payload) = get_raw_packet(data, i);
        
        Box::new(Self {
            raw: raw,
            port: payload[0],
            payload: payload[1..payload.len()].to_vec(),
        })
    }
    
    fn get_raw(&self) -> Vec<u8> {
        self.raw.clone()
    }
    
    fn get_packet_spec(&self) -> PacketSpec {
        packet_spec!(INPUT_CHUNKS, "InputChunks", "Port number (1-indexed) + a variable number of input chunks for that port.\nEach chunk can vary in size depending on the controller type in use on the respective frame.\nRefer to transitions to know if any controller types change mid-playback.\nThese packets, and the input chunks therein, are in sequential order!\nTherefore, any following input packets are appended to the inputs contained in this one.\nInput values are usually in native format (usually active-low), refer to `inputmaps.txt` for details.")
    }
    fn formatted_payload(&self) -> String {
        let mut out = format!("Port #{}, Chunks: ", self.port);
        self.payload.iter().for_each(|byte| out.push_str(format!("{:02X} ", byte).as_str()));
        
        out
    }
}


////////////////////////////////////// Transition //////////////////////////////////////
#[derive(Default, Clone, Debug)]
pub struct Transition {
    raw: Vec<u8>,
    index: u32,
    kind: u8,
    payload: Option<Vec<u8>>,
}
impl Transition {
    pub fn new(index: u32, kind: u8, payload: Option<Vec<u8>>) -> Box<Self> {
        let mut raw_payload = Vec::new();
        to_bytes(index as usize, 4).iter().for_each(|byte| raw_payload.push(*byte));
        raw_payload.push(kind);
        if payload.is_some() {
            payload.clone().unwrap().iter().for_each(|byte| raw_payload.push(*byte));
        }
        
        Box::new(Self {
            raw: serialize_payload(TRANSITION.to_vec(), raw_payload),
            index: index,
            kind: kind,
            payload: payload,
        })
    }
}
impl Packet for Transition {
    fn parse(data: &Vec<u8>, i: &mut usize) -> Box<dyn Packet> {
        let (raw, payload) = get_raw_packet(data, i);
        let index = to_u32(&payload[0..=3]);
        let kind = payload[4];
        let mut n = None;
        if payload.get(5).is_some() {
            n = Some(payload[5..payload.len()].to_vec());
        }
        
        Box::new(Self {
            raw: raw,
            index: index,
            kind: kind,
            payload: n,
        })
    }
    
    fn get_raw(&self) -> Vec<u8> {
        self.raw.clone()
    }
    
    fn get_packet_spec(&self) -> PacketSpec {
        packet_spec!(TRANSITION, "Transition", "Defines a transition at a specific point in the TAS. First 4 bytes is the frame/index number (0-indexed) based on all inputs contained in all FE01 packets. Then 1 byte specifying the transition type. Followed by a variable number of bytes if applicable.")
    }
    fn formatted_payload(&self) -> String {
        let mut out = format!("Index: {}, Kind: {}", self.index, transition_lut(self.kind).unwrap_or(format!("Unknown ({:02X})", self.kind).as_str()));
        if self.kind == 0xFF && self.payload.is_some() {
            let n = self.payload.clone().unwrap();
            if n.len() < 2 + 2 {
                out.push_str(" from invalid packet: ");
                for byte in &n { out.push_str(format!("{:02X} ", byte).as_str()); }
            } else {
                let internal_packet = parse_packet(&n, &mut 0);
                out.push_str(format!(" from {}: {}", internal_packet.get_packet_spec().name, internal_packet.formatted_payload()).as_str());
            }
        }
        
        out
    }
}


////////////////////////////////////// LagFrameChunk //////////////////////////////////////
#[derive(Default, Clone, Debug)]
pub struct LagFrameChunk {
    raw: Vec<u8>,
    index: u32,
    length: u32,
}
impl LagFrameChunk {
    pub fn new(index: u32, length: u32) -> Box<Self> {
        let mut raw_payload = Vec::new();
        to_bytes(index as usize, 4).iter().for_each(|byte| raw_payload.push(*byte));
        to_bytes(length as usize, 4).iter().for_each(|byte| raw_payload.push(*byte));
        
        Box::new(Self {
            raw: serialize_payload(LAG_FRAME_CHUNK.to_vec(), raw_payload),
            index: index,
            length: length,
        })
    }
}
impl Packet for LagFrameChunk {
    fn parse(data: &Vec<u8>, i: &mut usize) -> Box<dyn Packet> {
        let (raw, payload) = get_raw_packet(data, i);
        let index = to_u32(&payload[0..=3]);
        let length = to_u32(&payload[4..=7]);
        
        Box::new(Self {
            raw: raw,
            index: index,
            length: length,
        })
    }
    
    fn get_raw(&self) -> Vec<u8> {
        self.raw.clone()
    }
    
    fn get_packet_spec(&self) -> PacketSpec {
        packet_spec!(LAG_FRAME_CHUNK, "LagFrameChunk", "Specifies a chunk of lag frames based on the original TAS movie. First 4 bytes is the frame number (0-indexed) this chunk starts on. Second 4 bytes is the number of sequential lag frames in this chunk.")
    }
    fn formatted_payload(&self) -> String {
        format!("Index: {}, Length: {}", self.index, self.length)
    }
}


////////////////////////////////////// MovieTransition //////////////////////////////////////
#[derive(Default, Clone, Debug)]
pub struct MovieTransition {
    raw: Vec<u8>,
    index: u32,
    kind: u8,
    payload: Option<Vec<u8>>,
}
impl MovieTransition {
    pub fn new(index: u32, kind: u8, payload: Option<Vec<u8>>) -> Box<Self> {
        let mut raw_payload = Vec::new();
        to_bytes(index as usize, 4).iter().for_each(|byte| raw_payload.push(*byte));
        raw_payload.push(kind);
        if payload.is_some() {
            payload.clone().unwrap().iter().for_each(|byte| raw_payload.push(*byte));
        }
        
        Box::new(Self {
            raw: serialize_payload(MOVIE_TRANSITION.to_vec(), raw_payload),
            index: index,
            kind: kind,
            payload: payload,
        })
    }
}
impl Packet for MovieTransition {
    fn parse(data: &Vec<u8>, i: &mut usize) -> Box<dyn Packet> {
        let (raw, payload) = get_raw_packet(data, i);
        let index = to_u32(&payload[0..=3]);
        let kind = payload[4];
        let mut n = None;
        if payload.get(5).is_some() {
            n = Some(payload[5..payload.len()].to_vec());
        }
        
        Box::new(Self {
            raw: raw,
            index: index,
            kind: kind,
            payload: n,
        })
    }
    
    fn get_raw(&self) -> Vec<u8> {
        self.raw.clone()
    }
    
    fn get_packet_spec(&self) -> PacketSpec {
        packet_spec!(MOVIE_TRANSITION, "MovieTransition", "Defines a transition based on the original TAS movie frames (including lag frames). Using this packet requires FE03 packets. First 4 bytes is the movie frame number (0-indexed). Then 1 byte specifying the transition type. Followed by a variable number of bytes if applicable.")
    }
    fn formatted_payload(&self) -> String {
        let mut out = format!("Index: {}, Kind: {}", self.index, transition_lut(self.kind).unwrap_or(format!("Unknown ({:02X})", self.kind).as_str()));
        if self.kind == 0xFF && self.payload.is_some() {
            let n = self.payload.clone().unwrap();
            if n.len() < 2 + 2 {
                out.push_str(" from invalid packet: ");
                for byte in &n { out.push_str(format!("{:02X} ", byte).as_str()); }
            } else {
                let internal_packet = parse_packet(&n, &mut 0);
                out.push_str(format!(" from {}: {}", internal_packet.get_packet_spec().name, internal_packet.formatted_payload()).as_str());
            }
        }
        
        out
    }
}


















/// Assumes `i` starts at beginning of packet.
fn get_payload_len(data: &Vec<u8>, i: usize) -> (usize, usize) {
    let mut len = 0usize;
    
    let exp = *data.get(i + 2).unwrap() as usize;
    let len_bytes = data.get((i + 3)..(i + 3 + exp)).unwrap();
    for byte in len_bytes {
        len <<= 8;
        len |= *byte as usize;
    }

    (exp, len)
}

/// Assumes `i` starts at beginning of packet.
fn get_raw_bytes(data: &Vec<u8>, i: usize, exp: usize, payload_len: usize) -> Vec<u8> {
    let packet_len: usize = 3 + exp + payload_len;
    
    data.get(i..(i + packet_len)).unwrap().clone().to_vec()
}

/// Assumes `i` starts at beginning of packet.
fn get_payload_bytes(data: &Vec<u8>, i: usize, exp: usize, payload_len: usize) -> Vec<u8> {
    let offset = i + 3 + exp;
    
    data.get(offset..(offset + payload_len)).unwrap().clone().to_vec()
}

/// Assumes `i` starts at beginning of packet. Will increment `i` depending on length of packet.
fn get_raw_packet(data: &Vec<u8>, i: &mut usize) -> (Vec<u8>, Vec<u8>) {
    let (exp, len) = get_payload_len(data, *i);
    let out = (get_raw_bytes(data, *i, exp, len), get_payload_bytes(data, *i, exp, len));
    *i += out.0.len();
    
    out
}

/// Generates a raw packet of bytes using a `key` and `payload`, automatically calculating and inserting
///   the exponent and payload length bytes where necessary.
fn serialize_payload(key: Vec<u8>, payload: Vec<u8>) -> Vec<u8> {
    let mut len_bytes = Vec::<u8>::new();
    let mut len = payload.len();
    while len != 0 {
        len_bytes.push((len & 0xFF) as u8);
        len >>= 8;
    }
    
    let mut out = Vec::new();
    key.iter().for_each(|b| out.push(*b));          // Key
    out.push(len_bytes.len() as u8);                // Exponent
    len_bytes.iter().for_each(|b| out.push(*b));    // Payload Length
    payload.iter().for_each(|b| out.push(*b));      // Payload
    
    out
}