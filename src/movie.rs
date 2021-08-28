
use crate::definitions::*;
use crate::util::{to_u16, to_bytes};
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct TasdMovie {
    pub version: u16,
    pub key_width: u8,
    pub packets: Vec<Box<dyn Packet>>,
    pub source_file: PathBuf,
}
impl Default for TasdMovie {
    fn default() -> Self { Self {
        version: to_u16(&LATEST_VERSION),
        key_width: 2,
        packets: vec![],
        source_file: Default::default()
    }}
}
impl TasdMovie {
    pub fn new(path: &PathBuf) -> Result<Self, String> {
        let data = &std::fs::read(path).unwrap();
        if &data[0..4] != MAGIC_NUMBER {
            return Err(String::from("Magic Number doesn't match. This file doesn't appear to be a TASD."));
        }
        
        let mut tasd = Self::default();
        tasd.version = to_u16(&data[4..=5]);
        tasd.key_width = data[6];
        tasd.source_file = path.clone();
        
        let mut i = 7;
        loop {
            if i >= data.len() { break; }
            
            tasd.packets.push(parse_packet(data, &mut i));
        }
        
        Ok(tasd)
    }
    
    pub fn dump(&self) -> Vec<u8> {
        let mut out = Vec::new();
        MAGIC_NUMBER.iter().for_each(|byte| out.push(*byte));
        to_bytes(self.version as usize, 2).iter().for_each(|byte| out.push(*byte));
        out.push(0x02);
        self.packets.iter().for_each(|packet| packet.get_raw().iter().for_each(|byte| out.push(*byte)));
        
        out
    }
    
    pub fn save(&self) {
        std::fs::write(self.source_file.clone(), self.dump()).unwrap();
    }
    
    /// Searches and returns references to all packets which match the provided key(s)
    pub fn search_by_key(&self, keys: Vec<[u8; 2]>) -> Vec<&Box<dyn Packet>> {
        self.packets.iter().filter(|packet| keys.contains(&packet.get_packet_spec().key)).map(|packet| packet).collect()
    }
}

pub fn parse_packet(data: &Vec<u8>, i: &mut usize) -> Box<dyn Packet> {
    let key = [data[*i], data[*i + 1]];
            
    match key {
        CONSOLE_TYPE => ConsoleType::parse(data, i),
        CONSOLE_REGION => ConsoleRegion::parse(data, i),
        GAME_TITLE => GameTitle::parse(data, i),
        AUTHOR => Author::parse(data, i),
        CATEGORY => Category::parse(data, i),
        EMULATOR_NAME => EmulatorName::parse(data, i),
        EMULATOR_VERSION => EmulatorVersion::parse(data, i),
        EMULATOR_CORE => EmulatorCore::parse(data, i),
        TAS_LAST_MODIFIED => TASLastModified::parse(data, i),
        DUMP_LAST_MODIFIED => DumpLastModified::parse(data, i),
        TOTAL_FRAMES => TotalFrames::parse(data, i),
        RERECORDS => Rerecords::parse(data, i),
        SOURCE_LINK => SourceLink::parse(data, i),
        BLANK_FRAMES => BlankFrames::parse(data, i),
        VERIFIED => Verified::parse(data, i),
        MEMORY_INIT => MemoryInit::parse(data, i),
        PORT_CONTROLLER => PortController::parse(data, i),
        LATCH_FILTER => LatchFilter::parse(data, i),
        CLOCK_FILTER => ClockFilter::parse(data, i),
        OVERREAD => Overread::parse(data, i),
        GAME_GENIE_CODE => GameGenieCode::parse(data, i),
        INPUT_CHUNKS => InputChunks::parse(data, i),
        INPUT_MOMENT => InputMoment::parse(data, i),
        TRANSITION => Transition::parse(data, i),
        LAG_FRAME_CHUNK => LagFrameChunk::parse(data, i),
        MOVIE_TRANSITION => MovieTransition::parse(data, i),
        _ => Unsupported::parse(data, i),
    }
}