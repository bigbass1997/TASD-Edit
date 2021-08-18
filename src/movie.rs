
use crate::definitions::*;
use crate::util::to_u16;

#[derive(Default, Clone, Debug)]
pub struct TasdMovie {
    pub version: u16,
    pub key_width: u8,
    pub packets: Vec<Box<dyn Packet>>,
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
            let key = [data[i], data[i + 1]];
            
            let packet = match key {
                CONSOLE_TYPE => ConsoleType::parse(data, &mut i),
                CONSOLE_REGION => ConsoleRegion::parse(data, &mut i),
                GAME_TITLE => GameTitle::parse(data, &mut i),
                AUTHOR => Author::parse(data, &mut i),
                CATEGORY => Category::parse(data, &mut i),
                EMULATOR_NAME => EmulatorName::parse(data, &mut i),
                EMULATOR_VERSION => EmulatorVersion::parse(data, &mut i),
                TAS_LAST_MODIFIED => TASLastModified::parse(data, &mut i),
                DUMP_LAST_MODIFIED => DumpLastModified::parse(data, &mut i),
                TOTAL_FRAMES => TotalFrames::parse(data, &mut i),
                RERECORDS => Rerecords::parse(data, &mut i),
                SOURCE_LINK => SourceLink::parse(data, &mut i),
                BLANK_FRAMES => BlankFrames::parse(data, &mut i),
                VERIFIED => Verified::parse(data, &mut i),
                MEMORY_INIT => MemoryInit::parse(data, &mut i),
                LATCH_FILTER => LatchFilter::parse(data, &mut i),
                CLOCK_FILTER => ClockFilter::parse(data, &mut i),
                OVERREAD => Overread::parse(data, &mut i),
                DPCM => Dpcm::parse(data, &mut i),
                GAME_GENIE_CODE => GameGenieCode::parse(data, &mut i),
                INPUT_CHUNKS => InputChunks::parse(data, &mut i),
                TRANSITION => Transition::parse(data, &mut i),
                LAG_FRAME_CHUNK => LagFrameChunk::parse(data, &mut i),
                _ => Unsupported::parse(data, &mut i),
            };
            
            tasd.packets.push(packet);
        }
        
        Ok(tasd)
    }
}