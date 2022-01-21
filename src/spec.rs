
use std::any::Any;
use std::cmp::min;
use std::fmt::{Debug, Display, Formatter};
use std::path::PathBuf;
use chrono::{TimeZone, Utc};
use crossterm::style::Stylize;
use dyn_clone::DynClone;
use crate::util::{format_slice_bin, format_slice_hex, to_bytes, to_i64, to_u16, to_u32, to_u64, to_usize};


pub const LATEST_VERSION: [u8; 2] = [0x00, 0x01];
pub const MAGIC_NUMBER: [u8; 4] = [0x54, 0x41, 0x53, 0x44];
pub const NEW_TASD_FILE: [u8; 7] = [0x54, 0x41, 0x53, 0x44, LATEST_VERSION[0], LATEST_VERSION[1], 0x02];

pub type Key = [u8; 2];

pub const KEY_CONSOLE_TYPE: Key =       [0x00, 0x01];
pub const KEY_CONSOLE_REGION: Key =     [0x00, 0x02];
pub const KEY_GAME_TITLE: Key =         [0x00, 0x03];
pub const KEY_ROM_NAME: Key =           [0x00, 0x04];
pub const KEY_ATTRIBUTION: Key =        [0x00, 0x05];
pub const KEY_CATEGORY: Key =           [0x00, 0x06];
pub const KEY_EMULATOR_NAME: Key =      [0x00, 0x07];
pub const KEY_EMULATOR_VERSION: Key =   [0x00, 0x08];
pub const KEY_EMULATOR_CORE: Key =      [0x00, 0x09];
pub const KEY_TAS_LAST_MODIFIED: Key =  [0x00, 0x0A];
pub const KEY_DUMP_CREATED: Key =       [0x00, 0x0B];
pub const KEY_DUMP_LAST_MODIFIED: Key = [0x00, 0x0C];
pub const KEY_TOTAL_FRAMES: Key =       [0x00, 0x0D];
pub const KEY_RERECORDS: Key =          [0x00, 0x0E];
pub const KEY_SOURCE_LINK: Key =        [0x00, 0x0F];
pub const KEY_BLANK_FRAMES: Key =       [0x00, 0x10];
pub const KEY_VERIFIED: Key =           [0x00, 0x11];
pub const KEY_MEMORY_INIT: Key =        [0x00, 0x12];
pub const KEY_GAME_IDENTIFIER: Key =    [0x00, 0x13];
pub const KEY_MOVIE_LICENSE: Key =      [0x00, 0x14];
pub const KEY_MOVIE_FILE: Key =         [0x00, 0x15];

pub const KEY_PORT_CONTROLLER: Key =    [0x00, 0xF0];

pub const KEY_NES_LATCH_FILTER: Key =   [0x01, 0x01];
pub const KEY_NES_CLOCK_FILTER: Key =   [0x01, 0x02];
pub const KEY_NES_OVERREAD: Key =       [0x01, 0x03];
pub const KEY_NES_GAME_GENIE_CODE: Key = [0x01, 0x04];

pub const KEY_SNES_CLOCK_FILTER: Key =  [0x02, 0x02];
pub const KEY_SNES_OVERREAD: Key =      [0x02, 0x03];
pub const KEY_SNES_GAME_GENIE_CODE: Key = [0x02, 0x04];

pub const KEY_GENESIS_GAME_GENIE_CODE: Key = [0x08, 0x04];

pub const KEY_INPUT_CHUNK: Key =       [0xFE, 0x01];
pub const KEY_INPUT_MOMENT: Key =       [0xFE, 0x02];
pub const KEY_TRANSITION: Key =         [0xFE, 0x03];
pub const KEY_LAG_FRAME_CHUNK: Key =    [0xFE, 0x04];
pub const KEY_MOVIE_TRANSITION: Key =   [0xFE, 0x05];

pub const KEY_COMMENT: Key =            [0xFF, 0x01];
pub const KEY_UNSPECIFIED: Key =        [0xFF, 0xFF];

pub fn get_keys() -> Vec<(Key, &'static str, &'static str)> {
    vec![
        (KEY_CONSOLE_TYPE,              "CONSOLE_TYPE",             "The console used for this TAS"),
        (KEY_CONSOLE_REGION,            "CONSOLE_REGION",           "Video region of console"),
        (KEY_GAME_TITLE,                "GAME_TITLE",               "Name of the game used for this TAS"),
        (KEY_ROM_NAME,                  "ROM_NAME",                 "Specific ROM file name of the game used for this TAS"),
        (KEY_ATTRIBUTION,               "ATTRIBUTION",              "Name of a single person that contributed to the TAS, this file, or the verification"),
        (KEY_CATEGORY,                  "CATEGORY",                 "Category or branch name of this TAS"),
        (KEY_EMULATOR_NAME,             "EMULATOR_NAME",            "Name of the emulator used to dump this TAS"),
        (KEY_EMULATOR_VERSION,          "EMULATOR_VERSION",         "Version of the emulator used to dump this TAS"),
        (KEY_EMULATOR_CORE,             "EMULATOR_CORE",            "Name of the emulator core used to dump this TAS"),
        (KEY_TAS_LAST_MODIFIED,         "TAS_LAST_MODIFIED",        "The last modification date/time for the TAS movie this dump came from"),
        (KEY_DUMP_CREATED,              "DUMP_CREATED",             "The date/time this TASD file was first created"),
        (KEY_DUMP_LAST_MODIFIED,        "DUMP_LAST_MODIFIED",       "The last modification date/time for this TASD file"),
        (KEY_TOTAL_FRAMES,              "TOTAL_FRAMES",             "Total number of frames in the original TAS movie"),
        (KEY_RERECORDS,                 "RERECORDS",                "Rerecord count of the TAS movie"),
        (KEY_SOURCE_LINK,               "SOURCE_LINK",              "URL to the original movie's source/publication"),
        (KEY_BLANK_FRAMES,              "BLANK_FRAMES",             "Number of blank frames to prepend to the inputs of this TAS"),
        (KEY_VERIFIED,                  "VERIFIED",                 "Whether this TAS has been verified to work on real hardware"),
        (KEY_MEMORY_INIT,               "MEMORY_INIT",              "Specifies initial memory values used with this TAS"),
        (KEY_GAME_IDENTIFIER,           "GAME_IDENTIFIER",          "Unique identifier of the game used with this TAS (usually a hash or checksum)"),
        (KEY_MOVIE_LICENSE,             "MOVIE_LICENSE",            "Specifies what copyright license is used for the TAS movie"),
        (KEY_MOVIE_FILE,                "MOVIE_FILE",               "Embedded TAS movie file"),
        (KEY_PORT_CONTROLLER,           "PORT_CONTROLLER",          "The controller type used in a specific port during replay"),
        (KEY_NES_LATCH_FILTER,          "NES_LATCH_FILTER",         "A latch filter time in microseconds (1us) for how long the replay device should wait before advancing to the next input"),
        (KEY_NES_CLOCK_FILTER,          "NES_CLOCK_FILTER",         "A clock filter time in tenths of a microsecond (0.1us) for how long until new clock pulses should be accepted"),
        (KEY_NES_OVERREAD,              "NES_OVERREAD",             "Specifies whether a high (true) or low (false) signal is sent when the console requests more input buttons than expected"),
        (KEY_NES_GAME_GENIE_CODE,       "NES_GAME_GENIE_CODE",      "Game Genie code used for replay of this TAS"),
        (KEY_SNES_CLOCK_FILTER,         "SNES_CLOCK_FILTER",        "A clock filter time in tenths of a microsecond (0.1us) for how long until new clock pulses should be accepted"),
        (KEY_SNES_OVERREAD,             "SNES_OVERREAD",            "Specifies whether a high (true) or low (false) signal is sent when the console requests more input buttons than expected"),
        (KEY_SNES_GAME_GENIE_CODE,      "SNES_GAME_GENIE_CODE",     "Game Genie code used for replay of this TAS"),
        (KEY_GENESIS_GAME_GENIE_CODE,   "GENESIS_GAME_GENIE_CODE",  "Game Genie code used for replay of this TAS"),
        (KEY_INPUT_CHUNK,               "INPUT_CHUNK",              "A chunk of input data for a specific controller port. Controller data is structured based on the inputmaps.txt spec file"),
        (KEY_INPUT_MOMENT,              "INPUT_MOMENT",             "Input data for a specific controller port with specific timing. Controller data is structured based on the inputmaps.txt spec file"),
        (KEY_TRANSITION,                "TRANSITION",               "Specifies a transition at a specific point in the TAS replay"),
        (KEY_LAG_FRAME_CHUNK,           "LAG_FRAME_CHUNK",          "Specifies a chunk of lag frames pulled from the original TAS movie (should be used in conjunction with INPUT_CHUNK)"),
        (KEY_MOVIE_TRANSITION,          "MOVIE_TRANSITION",         "Specifies a transition at a specific point in the original TAS movie (will likely require LAG_FRAME_CHUNK's)"),
        (KEY_COMMENT,                   "COMMENT",                  "A string of text, often used for commentary (extra useful when combined with TRANSITION's)"),
        (KEY_UNSPECIFIED,               "UNSPECIFIED",              "Unspecified, arbitrary data"),
    ]
}

pub enum DumpError {
    StdError(std::io::Error),
    StdIo(std::io::ErrorKind),
    InvalidMagic,
    Custom(String),
}
impl Debug for DumpError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            InvalidMagic => f.write_str("Magic Number doesn't match TASD format."),
            _ => f.write_str(&format!("{:?}", self))
        }
    }
}
use DumpError::*;
use crate::lookup::{attribution_lut, console_region_lut, console_type_lut, controller_type_lut, game_identifier_lut, input_moment_lut, memory_init_lut, transition_index_lut, transition_kind_lut};


#[derive(Clone, Debug)]
pub struct TasdMovie {
    pub version: u16,
    pub keylen: u8,
    pub packets: Vec<Box<dyn Packet>>,
    pub source_path: PathBuf,
}
impl Default for TasdMovie {
    fn default() -> Self { Self {
        version: to_u16(&LATEST_VERSION),
        keylen: 2,
        packets: vec![],
        source_path: Default::default()
    }}
}
impl TasdMovie {
    pub fn new(path: &PathBuf) -> Result<Self, DumpError> {
        if !path.exists() {
            println!("Creating new TASD file at {}\n", path.canonicalize().unwrap_or(path.clone()).to_string_lossy());
            let mut tasd = Self::default();
            tasd.source_path = path.to_owned();
            tasd.save().unwrap();
            
            return Ok(tasd);
        }
        let data = match std::fs::read(path) {
            Ok(data) => data,
            Err(err) => return Err(StdError(err))
        };
        
        Ok(Self {
            version: to_u16(&LATEST_VERSION),
            keylen: 2,
            packets: parse_file(&data),
            source_path: path.to_owned()
        })
    }
    
    pub fn dump(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&MAGIC_NUMBER);
        out.extend_from_slice(&self.version.to_be_bytes());
        out.push(0x02);
        for packet in &self.packets {
            out.append(&mut packet.raw());
        }
        
        out
    }
    
    pub fn save(&mut self) -> std::io::Result<()> {
        let mut exists = false;
        let mut removals = Vec::new();
        for (i, packet) in self.packets.iter_mut().enumerate() {
            match packet.key() {
                KEY_DUMP_LAST_MODIFIED => {
                    if exists {
                        removals.push(i);
                    } else {
                        let epoch = Utc::now().timestamp();
                        *packet = Box::new(DumpLastModified::new(epoch));
                        exists = true;
                    }
                },
                _ => ()
            }
        }
        removals.iter().for_each(|i| { self.packets.remove(*i); });
        
        if !exists {
            let epoch = Utc::now().timestamp();
            let packet = Box::new(DumpLastModified::new(epoch));
            self.packets.push(packet);
        }
        
        std::fs::write(self.source_path.clone(), self.dump())
    }
    
    /// Searches and returns references to all packets which match the provided key(s)
    pub fn search_by_key(&self, keys: Vec<[u8; 2]>) -> Vec<&Box<dyn Packet>> {
        self.packets.iter().filter(|packet| keys.contains(&packet.key())).map(|packet| packet).collect()
    }
}

fn parse_file(data: &Vec<u8>) -> Vec<Box<dyn Packet>> {
    let mut packets = vec![];
    
    let mut i = 7;
    while i < data.len() {
        let exp = data[i + 2] as usize;
        let len = 3 + exp + to_usize(&data[(i + 3)..(i + 3 + exp)]);
        let chunk = &data[i..(i + len)];
        i += len;
        
        packets.push(parse_packet(chunk));
    }
    
    packets
}

fn parse_packet(data_chunk: &[u8]) -> Box<dyn Packet> {
    let key = [data_chunk[0], data_chunk[1]];
    let payload = parse_payload(data_chunk);
    
    match key {
        KEY_CONSOLE_TYPE => ConsoleType::parse(key, payload),
        KEY_CONSOLE_REGION => ConsoleRegion::parse(key, payload),
        KEY_GAME_TITLE => GameTitle::parse(key, payload),
        KEY_ROM_NAME => RomName::parse(key, payload),
        KEY_ATTRIBUTION => Attribution::parse(key, payload),
        KEY_CATEGORY => Category::parse(key, payload),
        KEY_EMULATOR_NAME => EmulatorName::parse(key, payload),
        KEY_EMULATOR_VERSION => EmulatorVersion::parse(key, payload),
        KEY_EMULATOR_CORE => EmulatorCore::parse(key, payload),
        KEY_TAS_LAST_MODIFIED => TasLastModified::parse(key, payload),
        KEY_DUMP_CREATED => DumpCreated::parse(key, payload),
        KEY_DUMP_LAST_MODIFIED => DumpLastModified::parse(key, payload),
        KEY_TOTAL_FRAMES => TotalFrames::parse(key, payload),
        KEY_RERECORDS => Rerecords::parse(key, payload),
        KEY_SOURCE_LINK => SourceLink::parse(key, payload),
        KEY_BLANK_FRAMES => BlankFrames::parse(key, payload),
        KEY_VERIFIED => Verified::parse(key, payload),
        KEY_MEMORY_INIT => MemoryInit::parse(key, payload),
        KEY_GAME_IDENTIFIER => GameIdentifier::parse(key, payload),
        KEY_MOVIE_LICENSE => MovieLicense::parse(key, payload),
        KEY_MOVIE_FILE => MovieFile::parse(key, payload),
        KEY_PORT_CONTROLLER => PortController::parse(key, payload),
        KEY_NES_LATCH_FILTER => NesLatchFilter::parse(key, payload),
        KEY_NES_CLOCK_FILTER => NesClockFilter::parse(key, payload),
        KEY_NES_OVERREAD => NesOverread::parse(key, payload),
        KEY_NES_GAME_GENIE_CODE => NesGameGenieCode::parse(key, payload),
        KEY_SNES_CLOCK_FILTER => SnesClockFilter::parse(key, payload),
        KEY_SNES_OVERREAD => SnesOverread::parse(key, payload),
        KEY_SNES_GAME_GENIE_CODE => SnesGameGenieCode::parse(key, payload),
        KEY_GENESIS_GAME_GENIE_CODE => GenesisGameGenieCode::parse(key, payload),
        KEY_INPUT_CHUNK => InputChunk::parse(key, payload),
        KEY_INPUT_MOMENT => InputMoment::parse(key, payload),
        KEY_TRANSITION => Transition::parse(key, payload),
        KEY_LAG_FRAME_CHUNK => LagFrameChunk::parse(key, payload),
        KEY_MOVIE_TRANSITION => MovieTransition::parse(key, payload),
        KEY_COMMENT => Comment::parse(key, payload),
        KEY_UNSPECIFIED => Unspecified::parse(key, payload),
        _ => Unsupported::parse(key, payload)
    }
}

fn parse_prefix_len(data_chunk: &[u8]) -> usize {
    3 + data_chunk[2] as usize
}

fn parse_payload(data_chunk: &[u8]) -> &[u8] {
    let i = parse_prefix_len(data_chunk);
    &data_chunk[i..]
}

fn payload_to_raw(key: Key, payload: &[u8]) -> Vec<u8> {
    let mut tmp = payload.len();
    if tmp == 0 {
        return vec![];
    }
    
    let mut exp = 0u8;
    while tmp > 0 {
        tmp >>= 8;
        exp += 1;
    }
    
    let mut raw = vec![key[0], key[1], exp];
    raw.append(&mut to_bytes(payload.len(), exp));
    raw.extend_from_slice(payload);
    
    raw
}

pub trait Packet: DynClone + Display {
    fn parse(key: Key, payload: &[u8]) -> Box<dyn Packet> where Self: Sized;
    fn raw(&self) -> Vec<u8>;
    fn key(&self) -> Key;
    
    fn as_any(&self) -> &dyn Any;
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
    pub key: Key,
    pub payload: Vec<u8>,
}
impl Unsupported {
    pub fn new(key: Key, data: &[u8]) -> Self { Self {
        key,
        payload: data.to_vec()
    }}
}
impl Display for Unsupported {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ...", "UNSUPPORTED".dark_yellow())
    }
}
impl Packet for Unsupported {
    fn parse(key: Key, payload: &[u8]) -> Box<dyn Packet> { Box::new(Self {
        key,
        payload: payload.to_vec()
    })}
    fn raw(&self) -> Vec<u8> {
        payload_to_raw(self.key, self.payload.as_slice())
    }
    fn key(&self) -> Key { self.key }
    fn as_any(&self) -> &dyn Any { self }
}

/*pub enum PacketTypes {
    ConsoleType { kind: u8, custom: Option<String> },
}
impl PacketTypes {
    pub fn parse(data: &[u8]) -> PacketTypes {
        PacketTypes::ConsoleType { kind: 0, custom: None }
    }
}*/

////////////////////////////////////// CONSOLE_TYPE //////////////////////////////////////
#[derive(Clone, Debug)]
pub struct ConsoleType {
    pub key: Key,
    pub kind: u8,
    pub custom: Option<String>,
}
impl ConsoleType {
    pub fn new(kind: u8, custom: Option<String>) -> Self { Self {
        key: KEY_CONSOLE_TYPE,
        kind,
        custom,
    }}
}
impl Display for ConsoleType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let kind = console_type_lut(self.kind).unwrap_or("Unknown");
        if let Some(custom) = &self.custom {
            write!(f, "{} {}: {}", "CONSOLE_TYPE".dark_yellow(), kind, custom)
        } else {
            write!(f, "{} {}", "CONSOLE_TYPE".dark_yellow(), kind)
        }
    }
}
impl Packet for ConsoleType {
    fn parse(key: Key, payload: &[u8]) -> Box<dyn Packet> { Box::new(Self {
        key,
        kind: payload[0],
        custom: if payload[0] == 0xFF { Some(String::from_utf8_lossy(&payload[1..]).to_string()) } else { None },
    })}
    fn raw(&self) -> Vec<u8> {
        payload_to_raw(self.key, &[&[self.kind], self.custom.as_ref().unwrap_or(&String::default()).as_bytes()].concat())
    }
    fn key(&self) -> Key { self.key }
    fn as_any(&self) -> &dyn Any { self }
}


////////////////////////////////////// CONSOLE_REGION //////////////////////////////////////
#[derive(Clone, Debug)]
pub struct ConsoleRegion {
    pub key: Key,
    pub region: u8,
}
impl ConsoleRegion {
    pub fn new(region: u8) -> Self { Self {
        key: KEY_CONSOLE_REGION,
        region,
    }}
}
impl Display for ConsoleRegion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", "CONSOLE_REGION".dark_yellow(), console_region_lut(self.region).unwrap_or("Unknown"))
    }
}
impl Packet for ConsoleRegion {
    fn parse(key: Key, payload: &[u8]) -> Box<dyn Packet> { Box::new(Self {
        key,
        region: payload[0],
    })}
    fn raw(&self) -> Vec<u8> {
        payload_to_raw(self.key, &[self.region])
    }
    fn key(&self) -> Key { self.key }
    fn as_any(&self) -> &dyn Any { self }
}


////////////////////////////////////// GAME_TITLE //////////////////////////////////////
#[derive(Clone, Debug)]
pub struct GameTitle {
    pub key: Key,
    pub title: String,
}
impl GameTitle {
    pub fn new(title: String) -> Self { Self {
        key: KEY_GAME_TITLE,
        title,
    }}
}
impl Display for GameTitle {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", "GAME_TITLE".dark_yellow(), self.title)
    }
}
impl Packet for GameTitle {
    fn parse(key: Key, payload: &[u8]) -> Box<dyn Packet> { Box::new(Self {
        key,
        title: String::from_utf8_lossy(payload).to_string(),
    })}
    fn raw(&self) -> Vec<u8> {
        payload_to_raw(self.key, self.title.as_bytes())
    }
    fn key(&self) -> Key { self.key }
    fn as_any(&self) -> &dyn Any { self }
}


////////////////////////////////////// ROM_NAME //////////////////////////////////////
#[derive(Clone, Debug)]
pub struct RomName {
    pub key: Key,
    pub name: String,
}
impl RomName {
    pub fn new(name: String) -> Self { Self {
        key: KEY_ROM_NAME,
        name,
    }}
}
impl Display for RomName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", "ROM_NAME".dark_yellow(), self.name)
    }
}
impl Packet for RomName {
    fn parse(key: Key, payload: &[u8]) -> Box<dyn Packet> { Box::new(Self {
        key,
        name: String::from_utf8_lossy(payload).to_string(),
    })}
    fn raw(&self) -> Vec<u8> {
        payload_to_raw(self.key, self.name.as_bytes())
    }
    fn key(&self) -> Key { self.key }
    fn as_any(&self) -> &dyn Any { self }
}


////////////////////////////////////// ATTRIBUTION //////////////////////////////////////
#[derive(Clone, Debug)]
pub struct Attribution {
    pub key: Key,
    pub kind: u8,
    pub name: String,
}
impl Attribution {
    pub fn new(kind: u8, name: String) -> Self { Self {
        key: KEY_ATTRIBUTION,
        kind,
        name,
    }}
}
impl Display for Attribution {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}: {}", "ATTRIBUTION".dark_yellow(), attribution_lut(self.kind).unwrap_or("Unknown"), self.name)
    }
}
impl Packet for Attribution {
    fn parse(key: Key, payload: &[u8]) -> Box<dyn Packet> { Box::new(Self {
        key,
        kind: payload[0],
        name: String::from_utf8_lossy(&payload[1..]).to_string(),
    })}
    fn raw(&self) -> Vec<u8> {
        payload_to_raw(self.key, &[&[self.kind], self.name.as_bytes()].concat())
    }
    fn key(&self) -> Key { self.key }
    fn as_any(&self) -> &dyn Any { self }
}


////////////////////////////////////// CATEGORY //////////////////////////////////////
#[derive(Clone, Debug)]
pub struct Category {
    pub key: Key,
    pub category: String,
}
impl Category {
    pub fn new(category: String) -> Self { Self {
        key: KEY_CATEGORY,
        category,
    }}
}
impl Display for Category {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", "CATEGORY".dark_yellow(), self.category)
    }
}
impl Packet for Category {
    fn parse(key: Key, payload: &[u8]) -> Box<dyn Packet> { Box::new(Self {
        key,
        category: String::from_utf8_lossy(payload).to_string(),
    })}
    fn raw(&self) -> Vec<u8> {
        payload_to_raw(self.key, self.category.as_bytes())
    }
    fn key(&self) -> Key { self.key }
    fn as_any(&self) -> &dyn Any { self }
}


////////////////////////////////////// EMULATOR_NAME //////////////////////////////////////
#[derive(Clone, Debug)]
pub struct EmulatorName {
    pub key: Key,
    pub name: String,
}
impl EmulatorName {
    pub fn new(name: String) -> Self { Self {
        key: KEY_EMULATOR_NAME,
        name,
    }}
}
impl Display for EmulatorName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", "EMULATOR_NAME".dark_yellow(), self.name)
    }
}
impl Packet for EmulatorName {
    fn parse(key: Key, payload: &[u8]) -> Box<dyn Packet> { Box::new(Self {
        key,
        name: String::from_utf8_lossy(payload).to_string(),
    })}
    fn raw(&self) -> Vec<u8> {
        payload_to_raw(self.key, self.name.as_bytes())
    }
    fn key(&self) -> Key { self.key }
    fn as_any(&self) -> &dyn Any { self }
}


////////////////////////////////////// EMULATOR_VERSION //////////////////////////////////////
#[derive(Clone, Debug)]
pub struct EmulatorVersion {
    pub key: Key,
    pub version: String,
}
impl EmulatorVersion {
    pub fn new(version: String) -> Self { Self {
        key: KEY_EMULATOR_VERSION,
        version,
    }}
}
impl Display for EmulatorVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", "EMULATOR_VERSION".dark_yellow(), self.version)
    }
}
impl Packet for EmulatorVersion {
    fn parse(key: Key, payload: &[u8]) -> Box<dyn Packet> { Box::new(Self {
        key,
        version: String::from_utf8_lossy(payload).to_string(),
    })}
    fn raw(&self) -> Vec<u8> {
        payload_to_raw(self.key, self.version.as_bytes())
    }
    fn key(&self) -> Key { self.key }
    fn as_any(&self) -> &dyn Any { self }
}


////////////////////////////////////// EMULATOR_CORE //////////////////////////////////////
#[derive(Clone, Debug)]
pub struct EmulatorCore {
    pub key: Key,
    pub core: String,
}
impl EmulatorCore {
    pub fn new(core: String) -> Self { Self {
        key: KEY_EMULATOR_CORE,
        core,
    }}
}
impl Display for EmulatorCore {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", "EMULATOR_CORE".dark_yellow(), self.core)
    }
}
impl Packet for EmulatorCore {
    fn parse(key: Key, payload: &[u8]) -> Box<dyn Packet> { Box::new(Self {
        key,
        core: String::from_utf8_lossy(payload).to_string(),
    })}
    fn raw(&self) -> Vec<u8> {
        payload_to_raw(self.key, self.core.as_bytes())
    }
    fn key(&self) -> Key { self.key }
    fn as_any(&self) -> &dyn Any { self }
}


////////////////////////////////////// TAS_LAST_MODIFIED //////////////////////////////////////
#[derive(Clone, Debug)]
pub struct TasLastModified {
    pub key: Key,
    pub epoch: i64,
}
impl TasLastModified {
    pub fn new(epoch: i64) -> Self { Self {
        key: KEY_TAS_LAST_MODIFIED,
        epoch,
    }}
}
impl Display for TasLastModified {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", "TAS_LAST_MODIFIED".dark_yellow(), Utc.timestamp(self.epoch, 0).to_string())
    }
}
impl Packet for TasLastModified {
    fn parse(key: Key, payload: &[u8]) -> Box<dyn Packet> { Box::new(Self {
        key,
        epoch: to_i64(payload),
    })}
    fn raw(&self) -> Vec<u8> {
        payload_to_raw(self.key, &self.epoch.to_be_bytes())
    }
    fn key(&self) -> Key { self.key }
    fn as_any(&self) -> &dyn Any { self }
}


////////////////////////////////////// DUMP_CREATED //////////////////////////////////////
#[derive(Clone, Debug)]
pub struct DumpCreated {
    pub key: Key,
    pub epoch: i64,
}
impl DumpCreated {
    pub fn new(epoch: i64) -> Self { Self {
        key: KEY_DUMP_CREATED,
        epoch,
    }}
}
impl Display for DumpCreated {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", "DUMP_CREATED".dark_yellow(), Utc.timestamp(self.epoch, 0).to_string())
    }
}
impl Packet for DumpCreated {
    fn parse(key: Key, payload: &[u8]) -> Box<dyn Packet> { Box::new(Self {
        key,
        epoch: to_i64(payload),
    })}
    fn raw(&self) -> Vec<u8> {
        payload_to_raw(self.key, &self.epoch.to_be_bytes())
    }
    fn key(&self) -> Key { self.key }
    fn as_any(&self) -> &dyn Any { self }
}


////////////////////////////////////// DUMP_LAST_MODIFIED //////////////////////////////////////
#[derive(Clone, Debug)]
pub struct DumpLastModified {
    pub key: Key,
    pub epoch: i64,
}
impl DumpLastModified {
    pub fn new(epoch: i64) -> Self { Self {
        key: KEY_DUMP_LAST_MODIFIED,
        epoch,
    }}
}
impl Display for DumpLastModified {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", "DUMP_LAST_MODIFIED".dark_yellow(), Utc.timestamp(self.epoch, 0).to_string())
    }
}
impl Packet for DumpLastModified {
    fn parse(key: Key, payload: &[u8]) -> Box<dyn Packet> { Box::new(Self {
        key,
        epoch: to_i64(payload),
    })}
    fn raw(&self) -> Vec<u8> {
        payload_to_raw(self.key, &self.epoch.to_be_bytes())
    }
    fn key(&self) -> Key { self.key }
    fn as_any(&self) -> &dyn Any { self }
}


////////////////////////////////////// TOTAL_FRAMES //////////////////////////////////////
#[derive(Clone, Debug)]
pub struct TotalFrames {
    pub key: Key,
    pub frames: u64,
}
impl TotalFrames {
    pub fn new(frames: u64) -> Self { Self {
        key: KEY_TOTAL_FRAMES,
        frames,
    }}
}
impl Display for TotalFrames {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", "TOTAL_FRAMES".dark_yellow(), self.frames)
    }
}
impl Packet for TotalFrames {
    fn parse(key: Key, payload: &[u8]) -> Box<dyn Packet> { Box::new(Self {
        key,
        frames: to_u64(payload),
    })}
    fn raw(&self) -> Vec<u8> {
        payload_to_raw(self.key, &self.frames.to_be_bytes())
    }
    fn key(&self) -> Key { self.key }
    fn as_any(&self) -> &dyn Any { self }
}


////////////////////////////////////// RERECORDS //////////////////////////////////////
#[derive(Clone, Debug)]
pub struct Rerecords {
    pub key: Key,
    pub rerecords: u64,
}
impl Rerecords {
    pub fn new(rerecords: u64) -> Self { Self {
        key: KEY_RERECORDS,
        rerecords,
    }}
}
impl Display for Rerecords {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", "RERECORDS".dark_yellow(), self.rerecords)
    }
}
impl Packet for Rerecords {
    fn parse(key: Key, payload: &[u8]) -> Box<dyn Packet> { Box::new(Self {
        key,
        rerecords: to_u64(payload),
    })}
    fn raw(&self) -> Vec<u8> {
        payload_to_raw(self.key, &self.rerecords.to_be_bytes())
    }
    fn key(&self) -> Key { self.key }
    fn as_any(&self) -> &dyn Any { self }
}


////////////////////////////////////// SOURCE_LINK //////////////////////////////////////
#[derive(Clone, Debug)]
pub struct SourceLink {
    pub key: Key,
    pub link: String,
}
impl SourceLink {
    pub fn new(link: String) -> Self { Self {
        key: KEY_SOURCE_LINK,
        link,
    }}
}
impl Display for SourceLink {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", "SOURCE_LINK".dark_yellow(), self.link)
    }
}
impl Packet for SourceLink {
    fn parse(key: Key, payload: &[u8]) -> Box<dyn Packet> { Box::new(Self {
        key,
        link: String::from_utf8_lossy(payload).to_string(),
    })}
    fn raw(&self) -> Vec<u8> {
        payload_to_raw(self.key, self.link.as_bytes())
    }
    fn key(&self) -> Key { self.key }
    fn as_any(&self) -> &dyn Any { self }
}


////////////////////////////////////// BLANK_FRAMES //////////////////////////////////////
#[derive(Clone, Debug)]
pub struct BlankFrames {
    pub key: Key,
    pub frames: u32,
}
impl BlankFrames {
    pub fn new(frames: u32) -> Self { Self {
        key: KEY_BLANK_FRAMES,
        frames,
    }}
}
impl Display for BlankFrames {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", "BLANK_FRAMES".dark_yellow(), self.frames)
    }
}
impl Packet for BlankFrames {
    fn parse(key: Key, payload: &[u8]) -> Box<dyn Packet> { Box::new(Self {
        key,
        frames: to_u32(payload),
    })}
    fn raw(&self) -> Vec<u8> {
        payload_to_raw(self.key, &self.frames.to_be_bytes())
    }
    fn key(&self) -> Key { self.key }
    fn as_any(&self) -> &dyn Any { self }
}


////////////////////////////////////// VERIFIED //////////////////////////////////////
#[derive(Clone, Debug)]
pub struct Verified {
    pub key: Key,
    pub verified: bool,
}
impl Verified {
    pub fn new(verified: bool) -> Self { Self {
        key: KEY_VERIFIED,
        verified,
    }}
}
impl Display for Verified {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", "VERIFIED".dark_yellow(), self.verified)
    }
}
impl Packet for Verified {
    fn parse(key: Key, payload: &[u8]) -> Box<dyn Packet> { Box::new(Self {
        key,
        verified: payload[0] != 0,
    })}
    fn raw(&self) -> Vec<u8> {
        payload_to_raw(self.key, &[self.verified as u8])
    }
    fn key(&self) -> Key { self.key }
    fn as_any(&self) -> &dyn Any { self }
}


////////////////////////////////////// MEMORY_INIT //////////////////////////////////////
#[derive(Clone, Debug)]
pub struct MemoryInit {
    pub key: Key,
    pub kind: u16,
    pub required: bool,
    pub name: String,
    pub data: Option<Vec<u8>>,
}
impl MemoryInit {
    pub fn new(kind: u16, required: bool, name: String, data: Option<Vec<u8>>) -> Self { Self {
        key: KEY_MEMORY_INIT,
        kind,
        required,
        name,
        data,
    }}
}
impl Display for MemoryInit {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}, Required? {}, Name: {}", "MEMORY_INIT".dark_yellow(), memory_init_lut(self.kind).unwrap_or("Unknown"), self.required, self.name)
    }
}
impl Packet for MemoryInit {
    fn parse(key: Key, payload: &[u8]) -> Box<dyn Packet> {
        let plen = payload[2] as usize;
        Box::new(Self {
            key,
            kind: to_u16(&payload[0..2]),
            required: payload[2] != 0,
            name: String::from_utf8_lossy(&payload[4..(4 + plen)]).to_string(),
            data: if (4 + plen) < payload.len() { Some(payload[(4 + plen)..].to_vec()) } else { None },
        })
    }
    fn raw(&self) -> Vec<u8> {
        payload_to_raw(self.key, &[&self.kind.to_be_bytes(), &[self.required as u8, self.name.len() as u8], &self.name.as_bytes()[..min(256, self.name.len())], self.data.as_ref().unwrap_or(&vec![]).as_slice()].concat())
    }
    fn key(&self) -> Key { self.key }
    fn as_any(&self) -> &dyn Any { self }
}


////////////////////////////////////// GAME_IDENTIFIER //////////////////////////////////////
#[derive(Clone, Debug)]
pub struct GameIdentifier {
    pub key: Key,
    pub kind: u8,
    pub hex: bool,
    pub base64: bool,
    pub identifier: Vec<u8>,
}
impl GameIdentifier {
    pub fn new(kind: u8, hex: bool, base64: bool, identifier: Vec<u8>) -> Self { Self {
        key: KEY_GAME_IDENTIFIER,
        kind,
        hex,
        base64,
        identifier,
    }}
}
impl Display for GameIdentifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}, Hex? {}, Base64? {}, Identifier: {}", "GAME_IDENTIFIER".dark_yellow(), game_identifier_lut(self.kind).unwrap_or("Unknown"), self.hex, self.base64, format_slice_hex(&self.identifier))
    }
}
impl Packet for GameIdentifier {
    fn parse(key: Key, payload: &[u8]) -> Box<dyn Packet> { Box::new(Self {
        key,
        kind: payload[0],
        hex: payload[1] != 0,
        base64: payload[2] != 0,
        identifier: payload[3..].to_vec(),
    })}
    fn raw(&self) -> Vec<u8> {
        payload_to_raw(self.key, &[&[self.kind, self.hex as u8, self.base64 as u8], self.identifier.as_slice()].concat())
    }
    fn key(&self) -> Key { self.key }
    fn as_any(&self) -> &dyn Any { self }
}


////////////////////////////////////// MOVIE_LICENSE //////////////////////////////////////
#[derive(Clone, Debug)]
pub struct MovieLicense {
    pub key: Key,
    pub license: String,
}
impl MovieLicense {
    pub fn new(license: String) -> Self { Self {
        key: KEY_MOVIE_LICENSE,
        license,
    }}
}
impl Display for MovieLicense {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", "MOVIE_LICENSE".dark_yellow(), self.license)
    }
}
impl Packet for MovieLicense {
    fn parse(key: Key, payload: &[u8]) -> Box<dyn Packet> { Box::new(Self {
        key,
        license: String::from_utf8_lossy(payload).to_string(),
    })}
    fn raw(&self) -> Vec<u8> {
        payload_to_raw(self.key, self.license.as_bytes())
    }
    fn key(&self) -> Key { self.key }
    fn as_any(&self) -> &dyn Any { self }
}


////////////////////////////////////// MOVIE_FILE //////////////////////////////////////
#[derive(Clone, Debug)]
pub struct MovieFile {
    pub key: Key,
    pub name: String,
    pub data: Vec<u8>,
}
impl MovieFile {
    pub fn new(name: String, data: Vec<u8>) -> Self { Self {
        key: KEY_MOVIE_FILE,
        name,
        data,
    }}
}
impl Display for MovieFile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} Name: {}", "MOVIE_FILE".dark_yellow(), self.name)
    }
}
impl Packet for MovieFile {
    fn parse(key: Key, payload: &[u8]) -> Box<dyn Packet> { Box::new(Self {
        key,
        name: String::from_utf8_lossy(&payload[1..(1 + payload[0] as usize)]).to_string(),
        data: payload[(1 + payload[0] as usize)..].to_vec(),
    })}
    fn raw(&self) -> Vec<u8> {
        payload_to_raw(self.key, &[&[self.name.len() as u8], &self.name.as_bytes()[..min(256, self.name.len())], self.data.as_slice()].concat())
    }
    fn key(&self) -> Key { self.key }
    fn as_any(&self) -> &dyn Any { self }
}


////////////////////////////////////// PORT_CONTROLLER //////////////////////////////////////
#[derive(Clone, Debug)]
pub struct PortController {
    pub key: Key,
    pub port: u8,
    pub kind: u16,
}
impl PortController {
    pub fn new(port: u8, kind: u16) -> Self { Self {
        key: KEY_PORT_CONTROLLER,
        port,
        kind,
    }}
}
impl Display for PortController {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} Port #{}: {}", "PORT_CONTROLLER".dark_yellow(), self.port, controller_type_lut(self.kind).unwrap_or("Unknown"))
    }
}
impl Packet for PortController {
    fn parse(key: Key, payload: &[u8]) -> Box<dyn Packet> { Box::new(Self {
        key,
        port: payload[0],
        kind: to_u16(&payload[1..3]),
    })}
    fn raw(&self) -> Vec<u8> {
        payload_to_raw(self.key, &[self.port, self.kind.to_be_bytes()[0], self.kind.to_be_bytes()[1]])
    }
    fn key(&self) -> Key { self.key }
    fn as_any(&self) -> &dyn Any { self }
}


////////////////////////////////////// NES_LATCH_FILTER //////////////////////////////////////
#[derive(Clone, Debug)]
pub struct NesLatchFilter {
    pub key: Key,
    pub time: u16,
}
impl NesLatchFilter {
    pub fn new(time: u16) -> Self { Self {
        key: KEY_NES_LATCH_FILTER,
        time,
    }}
}
impl Display for NesLatchFilter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}us", "NES_LATCH_FILTER".dark_yellow(), self.time)
    }
}
impl Packet for NesLatchFilter {
    fn parse(key: Key, payload: &[u8]) -> Box<dyn Packet> { Box::new(Self {
        key,
        time: to_u16(&payload[0..1]),
    })}
    fn raw(&self) -> Vec<u8> {
        payload_to_raw(self.key, &self.time.to_be_bytes())
    }
    fn key(&self) -> Key { self.key }
    fn as_any(&self) -> &dyn Any { self }
}


////////////////////////////////////// NES_CLOCK_FILTER //////////////////////////////////////
#[derive(Clone, Debug)]
pub struct NesClockFilter {
    pub key: Key,
    pub time: u8,
}
impl NesClockFilter {
    pub fn new(time: u8) -> Self { Self {
        key: KEY_NES_CLOCK_FILTER,
        time,
    }}
}
impl Display for NesClockFilter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {:.1}us", "NES_CLOCK_FILTER".dark_yellow(), self.time as f32 * 0.1)
    }
}
impl Packet for NesClockFilter {
    fn parse(key: Key, payload: &[u8]) -> Box<dyn Packet> { Box::new(Self {
        key,
        time: payload[0],
    })}
    fn raw(&self) -> Vec<u8> {
        payload_to_raw(self.key, &[self.time])
    }
    fn key(&self) -> Key { self.key }
    fn as_any(&self) -> &dyn Any { self }
}


////////////////////////////////////// NES_OVERREAD //////////////////////////////////////
#[derive(Clone, Debug)]
pub struct NesOverread {
    pub key: Key,
    pub overread: bool,
}
impl NesOverread {
    pub fn new(overread: bool) -> Self { Self {
        key: KEY_NES_OVERREAD,
        overread,
    }}
}
impl Display for NesOverread {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", "NES_OVERREAD".dark_yellow(), if self.overread { "HIGH" } else { "LOW" })
    }
}
impl Packet for NesOverread {
    fn parse(key: Key, payload: &[u8]) -> Box<dyn Packet> { Box::new(Self {
        key,
        overread: payload[0] != 0,
    })}
    fn raw(&self) -> Vec<u8> {
        payload_to_raw(self.key, &[self.overread as u8])
    }
    fn key(&self) -> Key { self.key }
    fn as_any(&self) -> &dyn Any { self }
}


////////////////////////////////////// NES_GAME_GENIE_CODE //////////////////////////////////////
#[derive(Clone, Debug)]
pub struct NesGameGenieCode {
    pub key: Key,
    pub code: String,
}
impl NesGameGenieCode {
    pub fn new(code: String) -> Self { Self {
        key: KEY_NES_GAME_GENIE_CODE,
        code,
    }}
}
impl Display for NesGameGenieCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", "NES_GAME_GENIE_CODE".dark_yellow(), self.code)
    }
}
impl Packet for NesGameGenieCode {
    fn parse(key: Key, payload: &[u8]) -> Box<dyn Packet> { Box::new(Self {
        key,
        code: String::from_utf8_lossy(payload).to_string(),
    })}
    fn raw(&self) -> Vec<u8> {
        payload_to_raw(self.key, self.code.as_bytes())
    }
    fn key(&self) -> Key { self.key }
    fn as_any(&self) -> &dyn Any { self }
}


////////////////////////////////////// SNES_CLOCK_FILTER //////////////////////////////////////
#[derive(Clone, Debug)]
pub struct SnesClockFilter {
    pub key: Key,
    pub time: u8,
}
impl SnesClockFilter {
    pub fn new(time: u8) -> Self { Self {
        key: KEY_SNES_CLOCK_FILTER,
        time,
    }}
}
impl Display for SnesClockFilter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {:.1}us", "SNES_CLOCK_FILTER".dark_yellow(), self.time as f32 * 0.1)
    }
}
impl Packet for SnesClockFilter {
    fn parse(key: Key, payload: &[u8]) -> Box<dyn Packet> { Box::new(Self {
        key,
        time: payload[0],
    })}
    fn raw(&self) -> Vec<u8> {
        payload_to_raw(self.key, &[self.time])
    }
    fn key(&self) -> Key { self.key }
    fn as_any(&self) -> &dyn Any { self }
}


////////////////////////////////////// SNES_OVERREAD //////////////////////////////////////
#[derive(Clone, Debug)]
pub struct SnesOverread {
    pub key: Key,
    pub overread: bool,
}
impl SnesOverread {
    pub fn new(overread: bool) -> Self { Self {
        key: KEY_SNES_OVERREAD,
        overread,
    }}
}
impl Display for SnesOverread {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", "SNES_OVERREAD".dark_yellow(), if self.overread { "HIGH" } else { "LOW" })
    }
}
impl Packet for SnesOverread {
    fn parse(key: Key, payload: &[u8]) -> Box<dyn Packet> { Box::new(Self {
        key,
        overread: payload[0] != 0,
    })}
    fn raw(&self) -> Vec<u8> {
        payload_to_raw(self.key, &[self.overread as u8])
    }
    fn key(&self) -> Key { self.key }
    fn as_any(&self) -> &dyn Any { self }
}


////////////////////////////////////// SNES_GAME_GENIE_CODE //////////////////////////////////////
#[derive(Clone, Debug)]
pub struct SnesGameGenieCode {
    pub key: Key,
    pub code: String,
}
impl SnesGameGenieCode {
    pub fn new(code: String) -> Self { Self {
        key: KEY_SNES_GAME_GENIE_CODE,
        code,
    }}
}
impl Display for SnesGameGenieCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", "SNES_GAME_GENIE_CODE".dark_yellow(), self.code)
    }
}
impl Packet for SnesGameGenieCode {
    fn parse(key: Key, payload: &[u8]) -> Box<dyn Packet> { Box::new(Self {
        key,
        code: String::from_utf8_lossy(payload).to_string(),
    })}
    fn raw(&self) -> Vec<u8> {
        payload_to_raw(self.key, self.code.as_bytes())
    }
    fn key(&self) -> Key { self.key }
    fn as_any(&self) -> &dyn Any { self }
}


////////////////////////////////////// GENESIS_GAME_GENIE_CODE //////////////////////////////////////
#[derive(Clone, Debug)]
pub struct GenesisGameGenieCode {
    pub key: Key,
    pub code: String,
}
impl GenesisGameGenieCode {
    pub fn new(code: String) -> Self { Self {
        key: KEY_GENESIS_GAME_GENIE_CODE,
        code,
    }}
}
impl Display for GenesisGameGenieCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", "GENESIS_GAME_GENIE_CODE".dark_yellow(), self.code)
    }
}
impl Packet for GenesisGameGenieCode {
    fn parse(key: Key, payload: &[u8]) -> Box<dyn Packet> { Box::new(Self {
        key,
        code: String::from_utf8_lossy(payload).to_string(),
    })}
    fn raw(&self) -> Vec<u8> {
        payload_to_raw(self.key, self.code.as_bytes())
    }
    fn key(&self) -> Key { self.key }
    fn as_any(&self) -> &dyn Any { self }
}


////////////////////////////////////// INPUT_CHUNK //////////////////////////////////////
#[derive(Clone, Debug)]
pub struct InputChunk {
    pub key: Key,
    pub port: u8,
    pub inputs: Vec<u8>,
}
impl InputChunk {
    pub fn new(port: u8, inputs: Vec<u8>) -> Self { Self {
        key: KEY_INPUT_CHUNK,
        port,
        inputs,
    }}
}
impl Display for InputChunk {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} Port #{} ...", "INPUT_CHUNK".dark_yellow(), self.port)
    }
}
impl Packet for InputChunk {
    fn parse(key: Key, payload: &[u8]) -> Box<dyn Packet> { Box::new(Self {
        key,
        port: payload[0],
        inputs: payload[1..].to_vec(),
    })}
    fn raw(&self) -> Vec<u8> {
        payload_to_raw(self.key, &[&[self.port], self.inputs.as_slice()].concat())
    }
    fn key(&self) -> Key { self.key }
    fn as_any(&self) -> &dyn Any { self }
}


////////////////////////////////////// INPUT_MOMENT //////////////////////////////////////
#[derive(Clone, Debug)]
pub struct InputMoment {
    pub key: Key,
    pub port: u8,
    pub kind: u8,
    pub index: u32,
    pub inputs: Vec<u8>,
}
impl InputMoment {
    pub fn new(port: u8, kind: u8, index: u32, inputs: Vec<u8>) -> Self { Self {
        key: KEY_INPUT_MOMENT,
        port,
        kind,
        index,
        inputs,
    }}
}
impl Display for InputMoment {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} Port #{}, {}, {} {}", "INPUT_MOMENT".dark_yellow(), self.port, input_moment_lut(self.kind).unwrap_or("Unknown"), self.index, format_slice_bin(&self.inputs))
    }
}
impl Packet for InputMoment {
    fn parse(key: Key, payload: &[u8]) -> Box<dyn Packet> { Box::new(Self {
        key,
        port: payload[0],
        kind: payload[1],
        index: to_u32(&payload[2..6]),
        inputs: payload[6..].to_vec(),
    })}
    fn raw(&self) -> Vec<u8> {
        let mut payload = vec![self.port];
        payload.push(self.kind);
        payload.extend_from_slice(&self.index.to_be_bytes());
        payload.extend_from_slice(&self.inputs);
        
        payload_to_raw(self.key, &payload)
    }
    fn key(&self) -> Key { self.key }
    fn as_any(&self) -> &dyn Any { self }
}


////////////////////////////////////// TRANSITION //////////////////////////////////////
#[derive(Clone, Debug)]
pub struct Transition {
    pub key: Key,
    pub index_kind: u8,
    pub index: u32,
    pub transition_kind: u8,
    pub packet: Option<Box<dyn Packet>>,
}
impl Transition {
    pub fn new(index_kind: u8, index: u32, transition_kind: u8, packet: Option<Box<dyn Packet>>) -> Self { Self {
        key: KEY_TRANSITION,
        index_kind,
        index,
        transition_kind,
        packet,
    }}
}
impl Display for Transition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}: {}, {} {}", "TRANSITION".dark_yellow(), transition_index_lut(self.index_kind).unwrap_or("Unknown"), self.index, transition_kind_lut(self.transition_kind).unwrap_or("Unknown"),
                if let Some(packet) = &self.packet {
                    packet.to_string()
                } else { String::new() }
        )
    }
}
impl Packet for Transition {
    fn parse(key: Key, payload: &[u8]) -> Box<dyn Packet> { Box::new(Self {
        key,
        index_kind: payload[0],
        index: to_u32(&payload[1..5]),
        transition_kind: payload[5],
        packet: if payload[5] == 0xFF { Some(parse_packet(&payload[6..])) } else { None },
    })}
    fn raw(&self) -> Vec<u8> {
        let mut payload = vec![self.index_kind];
        payload.extend_from_slice(&self.index.to_be_bytes());
        payload.push(self.transition_kind);
        if let Some(packet) = &self.packet {
            payload.append(&mut packet.raw());
        }
        
        payload_to_raw(self.key, &payload)
    }
    fn key(&self) -> Key { self.key }
    fn as_any(&self) -> &dyn Any { self }
}


////////////////////////////////////// LAG_FRAME_CHUNK //////////////////////////////////////
#[derive(Clone, Debug)]
pub struct LagFrameChunk {
    pub key: Key,
    pub frame: u32,
    pub count: u32,
}
impl LagFrameChunk {
    pub fn new(frame: u32, count: u32) -> Self { Self {
        key: KEY_LAG_FRAME_CHUNK,
        frame,
        count,
    }}
}
impl Display for LagFrameChunk {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} Start Frame #{}, Count: {}", "LAG_FRAME_CHUNK".dark_yellow(), self.frame, self.count)
    }
}
impl Packet for LagFrameChunk {
    fn parse(key: Key, payload: &[u8]) -> Box<dyn Packet> { Box::new(Self {
        key,
        frame: to_u32(&payload[0..4]),
        count: to_u32(&payload[4..8]),
    })}
    fn raw(&self) -> Vec<u8> {
        payload_to_raw(self.key, &[self.frame.to_be_bytes(), self.count.to_be_bytes()].concat())
    }
    fn key(&self) -> Key { self.key }
    fn as_any(&self) -> &dyn Any { self }
}


////////////////////////////////////// MOVIE_TRANSITION //////////////////////////////////////
#[derive(Clone, Debug)]
pub struct MovieTransition {
    pub key: Key,
    pub movie_frame: u32,
    pub transition_kind: u8,
    pub packet: Option<Box<dyn Packet>>,
}
impl MovieTransition {
    pub fn new(movie_frame: u32, transition_kind: u8, packet: Option<Box<dyn Packet>>) -> Self { Self {
        key: KEY_MOVIE_TRANSITION,
        movie_frame,
        transition_kind,
        packet,
    }}
}
impl Display for MovieTransition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} Movie Frame: {}, {} {}", "MOVIE_TRANSITION".dark_yellow(), self.movie_frame, transition_kind_lut(self.transition_kind).unwrap_or("Unknown"),
                if let Some(packet) = &self.packet {
                    packet.to_string()
                } else { String::new() }
        )
    }
}
impl Packet for MovieTransition {
    fn parse(key: Key, payload: &[u8]) -> Box<dyn Packet> { Box::new(Self {
        key,
        movie_frame: to_u32(&payload[0..4]),
        transition_kind: payload[4],
        packet: if payload[4] == 0xFF { Some(parse_packet(&payload[5..])) } else { None },
    })}
    fn raw(&self) -> Vec<u8> {
        let mut payload = vec![];
        payload.extend_from_slice(&self.movie_frame.to_be_bytes());
        payload.push(self.transition_kind);
        if let Some(packet) = &self.packet {
            payload.append(&mut packet.raw());
        }
        
        payload_to_raw(self.key, &payload)
    }
    fn key(&self) -> Key { self.key }
    fn as_any(&self) -> &dyn Any { self }
}


////////////////////////////////////// COMMENT //////////////////////////////////////
#[derive(Clone, Debug)]
pub struct Comment {
    pub key: Key,
    pub comment: String,
}
impl Comment {
    pub fn new(comment: String) -> Self { Self {
        key: KEY_COMMENT,
        comment,
    }}
}
impl Display for Comment {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", "COMMENT".dark_yellow(), self.comment)
    }
}
impl Packet for Comment {
    fn parse(key: Key, payload: &[u8]) -> Box<dyn Packet> { Box::new(Self {
        key,
        comment: String::from_utf8_lossy(payload).to_string(),
    })}
    fn raw(&self) -> Vec<u8> {
        payload_to_raw(self.key, self.comment.as_bytes())
    }
    fn key(&self) -> Key { self.key }
    fn as_any(&self) -> &dyn Any { self }
}


////////////////////////////////////// UNSPECIFIED //////////////////////////////////////
#[derive(Clone, Debug)]
pub struct Unspecified {
    pub key: Key,
    pub payload: Vec<u8>,
}
impl Unspecified {
    pub fn new(payload: Vec<u8>) -> Self { Self {
        key: KEY_UNSPECIFIED,
        payload,
    }}
}
impl Display for Unspecified {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ...", "UNSPECIFIED".dark_yellow(),)
    }
}
impl Packet for Unspecified {
    fn parse(key: Key, payload: &[u8]) -> Box<dyn Packet> { Box::new(Self {
        key,
        payload: payload.to_vec(),
    })}
    fn raw(&self) -> Vec<u8> {
        payload_to_raw(self.key, &self.payload)
    }
    fn key(&self) -> Key { self.key }
    fn as_any(&self) -> &dyn Any { self }
}























