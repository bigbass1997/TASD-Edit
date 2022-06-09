use std::cmp::max;
use std::ffi::OsStr;
use std::io::{Error, stdout, Write};
use std::path::PathBuf;
use chrono::{Date, NaiveDate, NaiveTime, Utc};

use crossterm::execute;
use crossterm::terminal::{SetTitle};
use crossterm::style::Stylize;

use tasd_edit::lookup::*;
use tasd_edit::spec::*;
use clap::{App, AppSettings, Arg};
use tasd_edit::util::{to_bytes, to_u16};


fn main() {
    execute!(stdout(), SetTitle("TASD-Edit")).unwrap();
    
    let matches = App::new("TASD-Edit")
        .arg(Arg::new("path")
            .takes_value(true)
            .help("Path to file to open. Optional. May be .tasd or any supported legacy format."))
        .setting(AppSettings::DeriveDisplayOrder)
        .setting(AppSettings::NextLineHelp)
        .get_matches();
    
    println!("");
    
    let mut tasd = None;
    
    if let Some(path) = matches.value_of("path") {
        let path = PathBuf::from(path);
        if path.is_dir() {
            println!("Path supplied is a directory; please specify a file instead.");
            exit(true, 0);
        }
        
        if path.is_file() {
            let data = std::fs::read(&path).unwrap();
            let ext = path.extension().unwrap_or(OsStr::new("")).to_string_lossy();
            if data[0..4] == MAGIC_NUMBER || ext == "tasd" {
                tasd = Some(TasdMovie::new(&path).unwrap());
            } else { match ext.as_ref() {
                "r08" | "r16m" | "txt" => match import_legacy(&mut tasd, Some(&path)) {
                    Ok(()) => (),
                    Err(err) => { println!("Err: {}", err); exit(true, 0); }
                },
                _ => { println!("Unable to determine what kind of file this is. Make sure it is a TASD file or has a supported legacy extention (.r08, .r16m, .txt)."); }
            }}
        } else {
            match path.extension().unwrap_or(OsStr::new("")).to_string_lossy().as_ref() {
                "r08" | "r16m" | "txt" => match import_legacy(&mut tasd, Some(&path)) {
                    Ok(()) => (),
                    Err(err) => { println!("Err: {}", err); exit(true, 0); }
                },
                _ => {
                    tasd = Some(TasdMovie::new(&path).unwrap());
                }
            }
        }
    }
    
    while !main_menu(&mut tasd) {}
    
    exit(false, 0);
}

fn main_menu(tasd_option: &mut Option<TasdMovie>) -> bool {
    if tasd_option.is_some() {
        let tasd = tasd_option.as_mut().unwrap();
        let selection = cli_selection(&[
                "Exit/Quit",
                "Add a new packet",
                "Remove a packet",
                "Import data from TASVideos",
                "Display all packets",
                "Display all, except inputs",
                "Save prettified packets to file",
                "Create/load a different TASD file",
                "Import and append a legacy file",
                "Export to legacy file",
            ], Some("What would you like to do?\n"), Some("Option[0]: ")
        );
        
        let mut ret = false;
        match selection {
          //0 => exits program
            1 => { while !add_menu(tasd) {} },
            2 => { while !remove_menu(tasd) {} },
            3 => { import_tasvideos(tasd); }
            4 => { display_packets(tasd, false); },
            5 => { display_packets(tasd, true); },
            6 => { save_pretty(tasd); },
            7 => { match load_tasd() {
                Err(x) => println!("Err: {:?}\n", x),
                Ok(x) => *tasd = x,
            }},
            8 => { match import_legacy(tasd_option, None) {
                Err(x) => println!("Err: {}\n", x),
                Ok(_) => ()
            }},
            9 => { export_legacy(tasd) },
            
            _ => ret = true,
        };
        
        ret
    } else {
        let selection = cli_selection(&[
                "Exit/Quit",
                "Create/load a TASD file",
                "Import a legacy file",
            ], Some("What would you like to do?\n"), Some("Option[0]: ")
        );
        
        let mut ret = false;
        match selection {
          //0 => exits program
            1 => { match load_tasd() {
                Err(x) => println!("Err: {:?}\n", x),
                Ok(x) => *tasd_option = Some(x),
            }},
            2 => {
                match import_legacy(tasd_option, None) {
                    Err(x) => println!("Err: {}\n", x),
                    Ok(_) => ()
                }
            },
            
            _ => ret = true,
        };
        
        ret
    }
}

fn add_menu(tasd: &mut TasdMovie) -> bool {
    let create = create_packet(Some("Select the packet you'd like to add.\n"), Some(vec![KEY_DUMP_LAST_MODIFIED]));
    
    if let Some(packet) = create.1 {
        if packet.key() != [0x00, 0x00] { // if packet is supported...
            tasd.packets.push(packet);
            tasd.save().unwrap();
            println!("New packet added to file!\n");
        }
    }
    
    create.0
}

fn create_packet(pretext: Option<&str>, exclude: Option<Vec<[u8; 2]>>) -> (bool, Option<Box<dyn Packet>>){
    let exclude = exclude.unwrap_or(vec![]);
    let mut options = vec!["Return to add menu".to_owned()];
    
    let mut included_types = vec![];
    for (key, name, description) in get_keys() {
        if !exclude.contains(&key) {
            included_types.push((key, name, description));
            options.push(format!("{}: {}", name.dark_yellow(), description));
        }
    }
    let selection = cli_selection(&options.iter().map(|s| s.as_ref()).collect::<Vec<&str>>(), pretext, Some("Packet Type[0]: "));
    if selection == 0 { return (true, None); }
    
    let spec = &included_types[selection - 1];
    let packet: Box<dyn Packet> = match spec.0 {
        KEY_CONSOLE_TYPE => {
            let mut options = vec!["Return to add menu"];
            let mut kinds = Vec::new();
            for i in 1..=255 {
                if let Some(kind) = console_type_lut(i) { options.push(kind); kinds.push(i); }
            }
            let selection = cli_selection(&options, None, Some("Console Type[0]: "));
            if selection == 0 { return (false, None); }
            let kind = kinds[selection - 1];
            
            if kind == 0xFF {
                let text = cli_read(Some("Custom type: "));
                if text.is_err() { println!("Err: {:?}\n", text.err().unwrap()); return (false, None); }
                
                Box::new(ConsoleType::new(kind, Some(text.unwrap())))
            } else {
                Box::new(ConsoleType::new(kind, None))
            }
        },
        KEY_CONSOLE_REGION => {
            let mut options = vec!["Return to add menu"];
            let mut kinds = Vec::new();
            for i in 1..=255 {
                let s = console_region_lut(i);
                if s.is_some() { options.push(s.unwrap()); kinds.push(i); }
            }
            let selection = cli_selection(&options, None, Some("Console Region[0]: "));
            if selection == 0 { return (false, None); }
            Box::new(ConsoleRegion::new(kinds[selection - 1]))
        },
        KEY_GAME_TITLE => {
            let text = cli_read(Some("Game title: "));
            if text.is_err() { println!("Err: {:?}\n", text.err().unwrap()); return (false, None); }
            Box::new(GameTitle::new(text.unwrap()))
        },
        KEY_ROM_NAME => {
            let text = cli_read(Some("ROM filename: "));
            if text.is_err() { println!("Err: {:?}\n", text.err().unwrap()); return (false, None); }
            Box::new(RomName::new(text.unwrap()))
        }
        KEY_ATTRIBUTION => {
            let mut options = vec!["Return to add menu"];
            let mut kinds = Vec::new();
            for i in 1..=255 {
                if let Some(kind) = attribution_lut(i) { options.push(kind); kinds.push(i); }
            }
            let selection = cli_selection(&options, None, Some("Attribution Type[0]: "));
            if selection == 0 { return (false, None); }
            let kind = kinds[selection - 1];
            
            let text = cli_read(Some("Name: "));
            if text.is_err() { println!("Err: {:?}\n", text.err().unwrap()); return (false, None); }
            Box::new(Attribution::new(kind, text.unwrap()))
        },
        KEY_CATEGORY => {
            let text = cli_read(Some("Category: "));
            if text.is_err() { println!("Err: {:?}\n", text.err().unwrap()); return (false, None); }
            Box::new(Category::new(text.unwrap()))
        },
        KEY_EMULATOR_NAME => {
            let text = cli_read(Some("Emulator name: "));
            if text.is_err() { println!("Err: {:?}\n", text.err().unwrap()); return (false, None); }
            Box::new(EmulatorName::new(text.unwrap()))
        },
        KEY_EMULATOR_VERSION => {
            let text = cli_read(Some("Emulator version: "));
            if text.is_err() { println!("Err: {:?}\n", text.err().unwrap()); return (false, None); }
            Box::new(EmulatorVersion::new(text.unwrap()))
        },
        KEY_EMULATOR_CORE => {
            let text = cli_read(Some("Emulator core: "));
            if text.is_err() { println!("Err: {:?}\n", text.err().unwrap()); return (false, None); }
            Box::new(EmulatorCore::new(text.unwrap()))
        },
        KEY_TAS_LAST_MODIFIED => {
            let text = cli_read(Some("TAS last modified (epoch seconds, YYYY-MM-DD, or YYYY-MM-DD HH:MM:SS): "));
            if text.is_err() { println!("Err: {:?}\n", text.err().unwrap()); return (false, None); }
            let text = text.unwrap();
            
            let epoch;
            let mut parse_attempt = NaiveDate::parse_from_str(&text, "%Y-%m-%d %H:%M:%S");
            if parse_attempt.is_err() {
                parse_attempt = NaiveDate::parse_from_str(&text, "%Y-%m-%d");
            }
            if parse_attempt.is_ok() {
                let parsed = parse_attempt.unwrap();
                let date = Date::<Utc>::from_utc(parsed, Utc);
                epoch = date.and_time(NaiveTime::from_hms(0,0,0)).unwrap().timestamp();
            } else {
                let parse_attempt = text.parse();
                if parse_attempt.is_err() { println!("Err: {:?}\n", parse_attempt.err().unwrap()); return (false, None); }
                epoch = parse_attempt.unwrap();
            }
            Box::new(TasLastModified::new(epoch))
        },
        KEY_DUMP_CREATED => {
            let text = cli_read(Some("Dump created (epoch seconds, YYYY-MM-DD, or YYYY-MM-DD HH:MM:SS): "));
            if text.is_err() { println!("Err: {:?}\n", text.err().unwrap()); return (false, None); }
            let text = text.unwrap();
            
            let epoch;
            let mut parse_attempt = NaiveDate::parse_from_str(&text, "%Y-%m-%d %H:%M:%S");
            if parse_attempt.is_err() {
                parse_attempt = NaiveDate::parse_from_str(&text, "%Y-%m-%d");
            }
            if parse_attempt.is_ok() {
                let parsed = parse_attempt.unwrap();
                let date = Date::<Utc>::from_utc(parsed, Utc);
                epoch = date.and_time(NaiveTime::from_hms(0,0,0)).unwrap().timestamp();
            } else {
                let parse_attempt = text.parse();
                if parse_attempt.is_err() { println!("Err: {:?}\n", parse_attempt.err().unwrap()); return (false, None); }
                epoch = parse_attempt.unwrap();
            }
            Box::new(DumpCreated::new(epoch))
        },
        KEY_TOTAL_FRAMES => {
            let text = cli_read(Some("Total frames: "));
            if text.is_err() { println!("Err: {:?}\n", text.err().unwrap()); return (false, None); }
            let parse_attempt = text.unwrap().parse();
            if parse_attempt.is_err() { println!("Err: {:?}\n", parse_attempt.err().unwrap()); return (false, None); }
            Box::new(TotalFrames::new(parse_attempt.unwrap()))
        },
        KEY_RERECORDS => {
            let text = cli_read(Some("Rerecord count: "));
            if text.is_err() { println!("Err: {:?}\n", text.err().unwrap()); return (false, None); }
            let parse_attempt = text.unwrap().parse();
            if parse_attempt.is_err() { println!("Err: {:?}\n", parse_attempt.err().unwrap()); return (false, None); }
            Box::new(Rerecords::new(parse_attempt.unwrap()))
        },
        KEY_SOURCE_LINK => {
            let text = cli_read(Some("Source link/url: "));
            if text.is_err() { println!("Err: {:?}\n", text.err().unwrap()); return (false, None); }
            Box::new(SourceLink::new(text.unwrap()))
        },
        KEY_BLANK_FRAMES => {
            let text = cli_read(Some("Blank frames (-32768 to +32767): "));
            if text.is_err() { println!("Err: {:?}\n", text.err().unwrap()); return (false, None); }
            let parse_attempt = text.unwrap().parse();
            if parse_attempt.is_err() { println!("Err: {:?}\n", parse_attempt.err().unwrap()); return (false, None); }
            Box::new(BlankFrames::new(parse_attempt.unwrap()))
        },
        KEY_VERIFIED => {
            let text = cli_read(Some("Has been verified (true or false): "));
            if text.is_err() { println!("Err: {:?}\n", text.err().unwrap()); return (false, None); }
            let parse_attempt = text.unwrap().parse::<bool>();
            if parse_attempt.is_err() { println!("Err: {:?}\n", parse_attempt.err().unwrap()); return (false, None); }
            Box::new(Verified::new(parse_attempt.unwrap()))
        },
        KEY_MEMORY_INIT => {
            let mut options = Vec::new();
            let mut kinds = Vec::new();
            options.push("Return to add menu");
            for i in 1..=255 {
                let s = memory_init_data_lut(i);
                if s.is_some() { options.push(s.unwrap()); kinds.push(i); }
            }
            let selection = cli_selection(&options, None, Some("Initialization type[0]: "));
            if selection == 0 { return (false, None); }
            let data_kind = kinds[selection - 1];
            
            let mut options = Vec::new();
            let mut kinds = Vec::new();
            options.push("Return to add menu");
            for i in 1..=65535 {
                let s = memory_init_device_lut(i);
                if s.is_some() { options.push(s.unwrap()); kinds.push(i); }
            }
            let selection = cli_selection(&options, None, Some("Initialization type[0]: "));
            if selection == 0 { return (false, None); }
            let device_kind = kinds[selection - 1];
            
            let name = cli_read(Some("Name of memory space: "));
            if name.is_err() { println!("Err: {:?}\n", name.err().unwrap()); return (false, None); }
            
            let mut payload = None;
            if data_kind == 0xFF {
                let path = cli_read(Some("Path to file containing memory data: "));
                if path.is_err() { println!("Err: {:?}\n", path.err().unwrap()); return (false, None); }
                let path = PathBuf::from(path.unwrap());
                if !path.exists() || !path.is_file() { println!("Path either doesn't exist or isn't a file.\n"); return (false, None); }
                let data_result = std::fs::read(path);
                if data_result.is_err() { println!("Err: {:?}\n", data_result.err().unwrap()); return (false, None); }
                payload = Some(data_result.unwrap());
            }
            
            let required = cli_read(Some("Required for verification (true or false): "));
            if required.is_err() { println!("Err: {:?}\n", required.err().unwrap()); return (false, None); }
            let required = required.unwrap().parse::<bool>();
            if required.is_err() { println!("Err: {:?}\n", required.err().unwrap()); return (false, None); }
            
            Box::new(MemoryInit::new(data_kind, device_kind, required.unwrap(), name.unwrap(), payload))
        },
        KEY_GAME_IDENTIFIER => {
            let mut options = Vec::new();
            let mut kinds = Vec::new();
            options.push("Return to add menu");
            for i in 1..=0xFF {
                let s = game_identifier_lut(i);
                if s.is_some() { options.push(s.unwrap()); kinds.push(i); }
            }
            let selection = cli_selection(&options, None, Some("Identifier type[0]: "));
            if selection == 0 { return (false, None); }
            let kind = kinds[selection - 1];
            
            let hex = cli_read(Some("Is this a hexadecimal string? (true or false): "));
            if hex.is_err() { println!("Err: {:?}\n", hex.err().unwrap()); return (false, None); }
            let hex = hex.unwrap().parse::<bool>();
            if hex.is_err() { println!("Err: {:?}\n", hex.err().unwrap()); return (false, None); }
            let hex = hex.unwrap();
            
            let mut base64 = false;
            if !hex {
                let tmp = cli_read(Some("Is this a base64-encoded string? (true or false): "));
                if tmp.is_err() { println!("Err: {:?}\n", tmp.err().unwrap()); return (false, None); }
                let tmp = tmp.unwrap().parse::<bool>();
                if tmp.is_err() { println!("Err: {:?}\n", tmp.err().unwrap()); return (false, None); }
                base64 = tmp.unwrap();
            }
            
            let identifier = cli_read(Some("Identifier (represented in hexadecimal): "));
            if identifier.is_err() { println!("Err: {:?}\n", identifier.err().unwrap()); return (false, None); }
            let mut identifier = identifier.unwrap();
            identifier = identifier.replace(" ", "");
            let identifier = (0..identifier.len()).step_by(2).map(|i| u8::from_str_radix(&identifier[i..i + 2], 16).unwrap()).collect();
            
            Box::new(GameIdentifier::new(kind, hex, base64, identifier))
        },
        KEY_MOVIE_LICENSE => {
            let text = cli_read(Some("Movie license: "));
            if text.is_err() { println!("Err: {:?}\n", text.err().unwrap()); return (false, None); }
            Box::new(MovieLicense::new(text.unwrap()))
        },
        KEY_MOVIE_FILE => {
            let path = cli_read(Some("Path to movie file: "));
            if path.is_err() { println!("Err: {:?}\n", path.err().unwrap()); return (false, None); }
            let path = PathBuf::from(path.unwrap());
            if !path.exists() || !path.is_file() { println!("Path either doesn't exist or isn't a file.\n"); return (false, None); }
            let data_result = std::fs::read(path.clone());
            if data_result.is_err() { println!("Err: {:?}\n", data_result.err().unwrap()); return (false, None); }
            Box::new(MovieFile::new(path.file_name().unwrap().to_string_lossy().to_string(), data_result.unwrap()))
        },
        KEY_PORT_CONTROLLER => {
            let text = cli_read(Some("Port number (1-indexed): "));
            if text.is_err() { println!("Err: {:?}\n", text.err().unwrap()); return (false, None); }
            let parse_attempt = text.unwrap().parse();
            if parse_attempt.is_err() { println!("Err: {:?}\n", parse_attempt.err().unwrap()); return (false, None); }
            let port = parse_attempt.unwrap();
            
            let mut options = Vec::new();
            let mut kinds = Vec::new();
            options.push("Return to add menu");
            for i in 1..=0xFFFF {
                let s = controller_type_lut(i);
                if s.is_some() { options.push(s.unwrap()); kinds.push(i); }
            }
            let selection = cli_selection(&options, None, Some("Controller type[0]: "));
            if selection == 0 { return (false, None); }
            let kind = kinds[selection - 1];
            
            Box::new(PortController::new(port, kind))
        },
        
        KEY_NES_LATCH_FILTER => {
            let text = cli_read(Some("Latch filter (integer from 0-65535; which will be multiplied by 1.0us): "));
            if text.is_err() { println!("Err: {:?}\n", text.err().unwrap()); return (false, None); }
            let parse_attempt = text.unwrap().parse();
            if parse_attempt.is_err() { println!("Err: {:?}\n", parse_attempt.err().unwrap()); return (false, None); }
            Box::new(NesLatchFilter::new(parse_attempt.unwrap()))
        },
        KEY_NES_CLOCK_FILTER => {
            let text = cli_read(Some("Clock filter (integer from 0-255; which will be multiplied by 0.1us): "));
            if text.is_err() { println!("Err: {:?}\n", text.err().unwrap()); return (false, None); }
            let parse_attempt = text.unwrap().parse();
            if parse_attempt.is_err() { println!("Err: {:?}\n", parse_attempt.err().unwrap()); return (false, None); }
            Box::new(NesClockFilter::new(parse_attempt.unwrap()))
        },
        KEY_NES_OVERREAD => {
            let text = cli_read(Some("Overread (true or false; true = HIGH, false = LOW): "));
            if text.is_err() { println!("Err: {:?}\n", text.err().unwrap()); return (false, None); }
            let parse_attempt = text.unwrap().parse();
            if parse_attempt.is_err() { println!("Err: {:?}\n", parse_attempt.err().unwrap()); return (false, None); }
            Box::new(NesOverread::new(parse_attempt.unwrap()))
        },
        KEY_NES_GAME_GENIE_CODE => {
            let text = cli_read(Some("Game genie code: "));
            if text.is_err() { println!("Err: {:?}\n", text.err().unwrap()); return (false, None); }
            Box::new(NesGameGenieCode::new(text.unwrap()))
        },
        
        KEY_SNES_CLOCK_FILTER => {
            let text = cli_read(Some("Clock filter (integer from 0-255; which will be multiplied by 0.1us): "));
            if text.is_err() { println!("Err: {:?}\n", text.err().unwrap()); return (false, None); }
            let parse_attempt = text.unwrap().parse();
            if parse_attempt.is_err() { println!("Err: {:?}\n", parse_attempt.err().unwrap()); return (false, None); }
            Box::new(SnesClockFilter::new(parse_attempt.unwrap()))
        },
        KEY_SNES_OVERREAD => {
            let text = cli_read(Some("Overread (true or false; true = HIGH, false = LOW): "));
            if text.is_err() { println!("Err: {:?}\n", text.err().unwrap()); return (false, None); }
            let parse_attempt = text.unwrap().parse();
            if parse_attempt.is_err() { println!("Err: {:?}\n", parse_attempt.err().unwrap()); return (false, None); }
            Box::new(SnesOverread::new(parse_attempt.unwrap()))
        },
        KEY_SNES_GAME_GENIE_CODE => {
            let text = cli_read(Some("Game genie code: "));
            if text.is_err() { println!("Err: {:?}\n", text.err().unwrap()); return (false, None); }
            Box::new(SnesGameGenieCode::new(text.unwrap()))
        },
        
        KEY_GENESIS_GAME_GENIE_CODE => {
            let text = cli_read(Some("Game genie code: "));
            if text.is_err() { println!("Err: {:?}\n", text.err().unwrap()); return (false, None); }
            Box::new(GenesisGameGenieCode::new(text.unwrap()))
        },
        
        //TODO: INPUT_CHUNKS: Idea is to list all frames with an index, and let user specify before or after a specific index to insert a new packet.
        //TODO: INPUT_MOMENT: Much easier to support
        
        KEY_TRANSITION => {
            let mut options = Vec::new();
            let mut kinds = Vec::new();
            options.push("Return to add menu");
            for i in 1..=255 {
                let s = transition_index_lut(i);
                if s.is_some() { options.push(s.unwrap()); kinds.push(i); }
            }
            let selection = cli_selection(&options, None, Some("Index type[0]: "));
            if selection == 0 { return (false, None); }
            let index_kind = kinds[selection - 1];
            
            let index = cli_read(Some("Index value: "));
            if index.is_err() { println!("Err: {:?}\n", index.err().unwrap()); return (false, None); }
            let parse_attempt = index.unwrap().parse::<u32>();
            if parse_attempt.is_err() { println!("Err: {:?}\n", parse_attempt.err().unwrap()); return (false, None); }
            
            let mut options = Vec::new();
            let mut kinds = Vec::new();
            options.push("Return to add menu");
            for i in 1..=255 {
                let s = transition_kind_lut(i);
                if s.is_some() { options.push(s.unwrap()); kinds.push(i); }
            }
            let selection = cli_selection(&options, None, Some("Transition type[0]: "));
            if selection == 0 { return (false, None); }
            let transition_kind = kinds[selection - 1];
            
            let mut payload = None;
            if transition_kind == 0xFF {
                let create = create_packet(Some("Select a packet for this transition.\n"), Some(vec![KEY_DUMP_LAST_MODIFIED, KEY_INPUT_CHUNK, KEY_INPUT_MOMENT, KEY_TRANSITION, KEY_LAG_FRAME_CHUNK, KEY_MOVIE_TRANSITION]));
                if create.1.is_none() { return (false, None); }
                payload = create.1;
            }
            
            Box::new(Transition::new(index_kind, parse_attempt.unwrap(), transition_kind, payload))
        },
        KEY_LAG_FRAME_CHUNK => {
            let index = cli_read(Some("Movie frame number: "));
            if index.is_err() { println!("Err: {:?}\n", index.err().unwrap()); return (false, None); }
            let index = index.unwrap().parse::<u32>();
            if index.is_err() { println!("Err: {:?}\n", index.err().unwrap()); return (false, None); }
            
            let length = cli_read(Some("Length of chunk: "));
            if length.is_err() { println!("Err: {:?}\n", length.err().unwrap()); return (false, None); }
            let length = length.unwrap().parse::<u32>();
            if length.is_err() { println!("Err: {:?}\n", length.err().unwrap()); return (false, None); }
            Box::new(LagFrameChunk::new(index.unwrap(), length.unwrap()))
        },
        KEY_MOVIE_TRANSITION => {
            let index = cli_read(Some("Frame number: "));
            if index.is_err() { println!("Err: {:?}\n", index.err().unwrap()); return (false, None); }
            let parse_attempt = index.unwrap().parse::<u32>();
            if parse_attempt.is_err() { println!("Err: {:?}\n", parse_attempt.err().unwrap()); return (false, None); }
            
            let mut options = Vec::new();
            let mut kinds = Vec::new();
            options.push("Return to add menu");
            for i in 1..=255 {
                let s = transition_kind_lut(i);
                if s.is_some() { options.push(s.unwrap()); kinds.push(i); }
            }
            let selection = cli_selection(&options, None, Some("Transition type[0]: "));
            if selection == 0 { return (false, None); }
            let transition_kind = kinds[selection - 1];
            
            let mut payload = None;
            if transition_kind == 0xFF {
                let create = create_packet(Some("Select a packet for this transition.\n"), Some(vec![KEY_DUMP_LAST_MODIFIED, KEY_INPUT_CHUNK, KEY_INPUT_MOMENT, KEY_TRANSITION, KEY_LAG_FRAME_CHUNK, KEY_MOVIE_TRANSITION]));
                if create.1.is_none() { return (true, None); }
                payload = create.1;
            }
            Box::new(MovieTransition::new(parse_attempt.unwrap(), transition_kind, payload))
        },
        
        KEY_COMMENT => {
            let text = cli_read(Some("Comment: "));
            if text.is_err() { println!("Err: {:?}\n", text.err().unwrap()); return (false, None); }
            Box::new(Comment::new(text.unwrap()))
        },
        KEY_UNSPECIFIED => {
            let selection = cli_selection(&["Return to add menu", "Text string", "Embed a file"], None, Some("Specify type of data[0]: "));
            if selection == 0 { return (false, None); }
            
            let payload = match selection {
                1 => {
                    let text = cli_read(Some("Text: "));
                    if text.is_err() { println!("Err: {:?}\n", text.err().unwrap()); return (false, None); }
                    text.unwrap().as_bytes().to_vec()
                },
                2 => {
                    let path = cli_read(Some("Path to file containing arbitrary data: "));
                    if path.is_err() { println!("Err: {:?}\n", path.err().unwrap()); return (false, None); }
                    let path = PathBuf::from(path.unwrap());
                    if !path.exists() || !path.is_file() { println!("Path either doesn't exist or isn't a file.\n"); return (false, None); }
                    let data_result = std::fs::read(path.clone());
                    if data_result.is_err() { println!("Err: {:?}\n", data_result.err().unwrap()); return (false, None); }
                    data_result.unwrap()
                },
                _ => return (false, None)
            };
            Box::new(Unspecified::new(payload))
        },
        _ => { println!("Sorry, creating new packets of this type is currently unsupported.\n"); return (false, None); },
    };
    
    (false, Some(packet))
}

fn remove_menu(tasd: &mut TasdMovie) -> bool {
    let mut options = vec![String::from("Return to main menu")];
    for packet in &tasd.packets {
        options.push(format!("{}", packet));
    }
    
    let selection = cli_selection(&options.iter().map(|s| s as &str).collect::<Vec<&str>>(), Some("Select the packet you wish to remove.\n"), Some("Packet index[0]: "));
    if selection != 0 {
        println!("Packet removed.\n");
        tasd.packets.remove(selection - 1);
        tasd.save().unwrap();
        return false;
    }
    
    true
}

fn import_tasvideos(tasd: &mut TasdMovie) {
    unimplemented!()
}

fn display_packets(tasd: &TasdMovie, exclude_inputs: bool) {
    let pretty = prettify_packets(tasd);
    for packet in pretty {
        if exclude_inputs && (packet.contains("INPUT_CHUNK") || packet.contains("INPUT_MOMENT")) { continue; }
        println!("{}", packet);
    }
    println!("");
}

fn save_pretty(tasd: &TasdMovie) {
    let pretty = prettify_packets(tasd);
    let mut path = tasd.source_path.clone();
    path.set_extension("tasd.pretty.txt");
    let mut out = Vec::new();
    for line in pretty {
        format!("{}\n", line).as_bytes().iter().for_each(|byte| out.push(*byte));
    }
    
    let result = std::fs::write(path.clone(), out);
    if result.is_err() {
        println!("Err: {:?}\n", result.err());
    } else {
        println!("File saved to: {}\n", path.canonicalize().unwrap().to_string_lossy())
    }
}

fn prettify_packets(tasd: &TasdMovie) -> Vec<String> {
    let mut out = Vec::new();
    
    out.push(format!("Version: {:#06X}, Key Width: {}", tasd.version, tasd.keylen));
    let padding = ((tasd.packets.len() as f32).log10() as usize) + 1;
    for (i, packet) in tasd.packets.iter().enumerate() {
        out.push(format!("[{}]: {}", format!("{:padding$}", i, padding=padding).cyan(), packet));
    }
    
    out
}

fn load_tasd() -> Result<TasdMovie, DumpError> {
    let result = cli_read(Some("Provide the name for a new empty file, or the path to an existing file you wish to load.\nFile name: "));
    if result.is_ok() {
        let mut name = result.unwrap();
        if name.is_empty() {
            return Err(DumpError::Custom("Err: Empty input. You must create or load a file to use this software.".to_owned()));
        } else {
            if !name.ends_with(".tasd") { name.push_str(".tasd") }
            let mut path = PathBuf::from(name);
            check_tasd_exists_create(&mut path);
            TasdMovie::new(&path)
        }
    } else {
        return Err(DumpError::Custom(format!("Err: {:?}", result.err().unwrap())));
    }
}

fn import_legacy(tasd_option: &mut Option<TasdMovie>, path: Option<&PathBuf>) -> Result<(), String> {
    let path = if let Some(path) = path {
        path.to_owned()
    } else {
        let result = cli_read(Some("Path to .r08, .r16m, or GBI(.txt) file: "));
        if result.is_err() { return Err(format!("Err: {:?}", result.err())) }
        PathBuf::from(result.unwrap())
    };
    if !path.exists() || path.is_dir() { return Err("Err: File either doesn't exist or is a directory.".to_owned()) }
    if path.extension().is_none() { return Err("Err: Unable to identify file, make sure extension is correct.".to_owned())}
    let extension = path.extension().unwrap().to_string_lossy().to_string();
    
    if tasd_option.is_none() {
        let mut tasd = TasdMovie::default();
        tasd.source_path = path.clone();
        tasd.source_path.set_extension("tasd");
        *tasd_option = Some(tasd);
    }
    let tasd = tasd_option.as_mut().unwrap();
    
    match extension.as_str() {
        "r08" => {
            let result = std::fs::read(path);
            if result.is_err() { return Err(format!("Err: {:?}", result.err())) }
            let mut result = result.unwrap();
            if result.len() % 2 == 1 { result.push(0xFF) } // Shouldn't ever be misaligned, but is safer to double check
            tasd.packets.push(Box::new(ConsoleType::new(0x01, None)));
            tasd.packets.push(Box::new(PortController::new(1, 0x0101)));
            tasd.packets.push(Box::new(PortController::new(2, 0x0101)));
            
            let mut port1 = Vec::new();
            let mut port2 = Vec::new();
            for i in 0..(result.len() / 2) {
                port1.push(result[(i * 2)] ^ 0xFF);
                port2.push(result[(i * 2) + 1] ^ 0xFF);
            }
            tasd.packets.push(Box::new(InputChunk::new(1, port1)));
            tasd.packets.push(Box::new(InputChunk::new(2, port2)));
            
            tasd.save().unwrap();
            println!("Legacy file data has been imported and saved.\n");
            Ok(())
        },
        "r16m" => {
            let result = std::fs::read(path);
            if result.is_err() { return Err(format!("Err: {:?}", result.err())) }
            let mut result = result.unwrap();
            if result.len() % 2 == 1 { result.push(0) } // Shouldn't ever be misaligned, but is safer to double check
            if result.len() % 4 == 1 { result.push(0); result.push(0); } // Shouldn't ever be misaligned, but is safer to double check
            tasd.packets.push(Box::new(ConsoleType::new(0x02, None)));
            tasd.packets.push(Box::new(PortController::new(1, 0x0201)));
            tasd.packets.push(Box::new(PortController::new(2, 0x0201)));
            
            let mut port1 = Vec::new();
            let mut port2 = Vec::new();
            for i in 0..(result.len() / 4) {
                port1.push(result[(i * 4)] ^ 0xFF);
                port1.push(result[(i * 4) + 1] ^ 0xFF);
                port2.push(result[(i * 4) + 2] ^ 0xFF);
                port2.push(result[(i * 4) + 3] ^ 0xFF);
            }
            tasd.packets.push(Box::new(InputChunk::new(1, port1)));
            tasd.packets.push(Box::new(InputChunk::new(2, port2)));
            
            tasd.save().unwrap();
            println!("Legacy file data has been imported and saved.\n");
            Ok(())
        },
        "txt" => {
            let selection = cli_selection(&["GB", "GBC", "GBA"], Some("Which handheld is this for?\n"), Some("Handheld type[0]: "));
            let result = std::fs::read_to_string(path);
            if result.is_err() { return Err(format!("Err: {:?}", result.err())) }
            let result = result.unwrap();
            let result = result.lines();
            match selection {
                0 => {
                    tasd.packets.push(Box::new(ConsoleType::new(0x05, None)));
                    tasd.packets.push(Box::new(PortController::new(1, 0x0501)));
                },
                1 => {
                    tasd.packets.push(Box::new(ConsoleType::new(0x06, None)));
                    tasd.packets.push(Box::new(PortController::new(1, 0x0601)));
                },
                2 => {
                    tasd.packets.push(Box::new(ConsoleType::new(0x07, None)));
                    tasd.packets.push(Box::new(PortController::new(1, 0x0701)));
                },
                _ => ()
            }
            
            for line in result {
                let parts = line.split_once(' ');
                if parts.is_some() {
                    let parts = parts.unwrap();
                    let clock = u32::from_str_radix(parts.0, 16).unwrap();
                    let input = to_bytes(u16::from_str_radix(parts.1, 16).unwrap() as usize, 2);
                    match selection {
                        0 => { tasd.packets.push(Box::new(InputMoment::new(1, 0x02, clock, vec![input[1] ^ 0xFF]))) },
                        1 => { tasd.packets.push(Box::new(InputMoment::new(1, 0x02, clock, vec![input[1] ^ 0xFF]))) },
                        2 => { tasd.packets.push(Box::new(InputMoment::new(1, 0x02, clock, vec![input[0] ^ 0xFF, input[1] ^ 0xFF]))) },
                        _ => ()
                    }
                }
            }
            
            tasd.save().unwrap();
            println!("Legacy file data has been imported and saved.\n");
            Ok(())
        },
        _ => Err("Unable to identify file, make sure extension is correct.".to_owned())
    }
}

fn export_legacy(tasd: &TasdMovie) {
    let valid_types = [0x01, 0x02, 0x05, 0x06, 0x07];
    let search = tasd.search_by_key(vec![KEY_CONSOLE_TYPE]);
    let mut packets = Vec::new();
    for packet in search {
        let packet = packet.as_any().downcast_ref::<ConsoleType>().unwrap();
        if valid_types.contains(&packet.kind) {
            packets.push(packet);
        }
    }
    
    let console_type = match packets.len() {
        0 => { println!("Unable to determine what console this data is intended for. Please add a ConsoleType packet."); return; },
        1 => packets[0].kind,
        _ => {
            let mut options = vec!["Return to main menu"];
            for packet in &packets {
                if let Some(kind) = console_type_lut(packet.kind) {
                    options.push(kind);
                }
            }
            let selection = cli_selection(&options, Some("Multiple console types detected. Select which you're trying to export to."), Some("Console type[0]: "));
            if selection == 0 { return; }
            
            packets[selection - 1].kind 
        },
    };
    
    match console_type {
        0x01 => { // NES (.r08)
            let path = tasd.source_path.with_extension("export.r08");
            let search = tasd.search_by_key(vec![KEY_INPUT_CHUNK]);
            
            let mut port1 = Vec::new();
            let mut port2 = Vec::new();
            for packet in search {
                let chunk = packet.as_any().downcast_ref::<InputChunk>().unwrap();
                if chunk.port == 1 { chunk.inputs.iter().for_each(|byte| port1.push(*byte ^ 0xFF)) }
                if chunk.port == 2 { chunk.inputs.iter().for_each(|byte| port2.push(*byte ^ 0xFF)) }
            }
            
            // if they are different lengths, make up the difference with default inputs
            let max_len = max(port1.len(), port2.len());
            for _ in 0..(max_len - port1.len()) { port1.push(0xFF) }
            for _ in 0..(max_len - port2.len()) { port2.push(0xFF) }
            
            let mut out = Vec::new();
            for i in 0..max_len {
                out.push(port1[i]);
                out.push(port2[i]);
            }
            
            let result = std::fs::write(path.clone(), out);
            if result.is_err() { println!("Err: {:?}\n", result.err().unwrap()); return; }
            println!("Legacy file data has been exported to: {}\n", path.canonicalize().unwrap().to_string_lossy());
        },
        0x02 => { // SNES (.r16m)
            let path = tasd.source_path.with_extension("export.r16m");
            let search = tasd.search_by_key(vec![KEY_INPUT_CHUNK]);
            
            let mut port1 = Vec::new();
            let mut port2 = Vec::new();
            let mut port3 = Vec::new();
            let mut port4 = Vec::new();
            for packet in search {
                let chunk = packet.as_any().downcast_ref::<InputChunk>().unwrap();
                if chunk.port == 1 { chunk.inputs.iter().for_each(|byte| port1.push(*byte ^ 0xFF)) }
                if chunk.port == 2 { chunk.inputs.iter().for_each(|byte| port2.push(*byte ^ 0xFF)) }
                if chunk.port == 3 { chunk.inputs.iter().for_each(|byte| port3.push(*byte ^ 0xFF)) }
                if chunk.port == 4 { chunk.inputs.iter().for_each(|byte| port4.push(*byte ^ 0xFF)) }
            }
            
            // if they are different lengths, make up the difference with default inputs
            let max_len = max(port1.len(), port2.len());
            for _ in 0..(max_len - port1.len()) { port1.push(0xFF) }
            for _ in 0..(max_len - port2.len()) { port2.push(0xFF) }
            
            let mut out = Vec::new();
            for i in 0..(max_len / 2) {
                out.push(port1[(i * 2)]);
                out.push(port1[(i * 2) + 1]);
                out.push(port2[(i * 2)]);
                out.push(port2[(i * 2) + 1]);
            }
            
            let result = std::fs::write(path.clone(), out);
            if result.is_err() { println!("Err: {:?}\n", result.err().unwrap()); return; }
            println!("Legacy file data has been exported to: {}\n", path.canonicalize().unwrap().to_string_lossy());
        },
        0x05 | 0x06 => { // GB/C (GBI .txt)
            let path = tasd.source_path.with_extension("export.txt");
            let search = tasd.search_by_key(vec![KEY_INPUT_MOMENT]);
            let mut out = Vec::new();
            for packet in search {
                let moment = packet.as_any().downcast_ref::<InputMoment>().unwrap();
                let line = format!("{:08X} {:04X}\n", moment.index, (moment.inputs[0] ^ 0xFF) as u16);
                
                line.as_bytes().iter().for_each(|byte| out.push(*byte));
            }
            
            let result = std::fs::write(path.clone(), out);
            if result.is_err() { println!("Err: {:?}\n", result.err().unwrap()); return; }
            println!("Legacy file data has been exported to: {}\n", path.canonicalize().unwrap().to_string_lossy());
        },
        0x07 => { // GBA (GBI .txt)
            let path = tasd.source_path.with_extension("export.txt");
            let search = tasd.search_by_key(vec![KEY_INPUT_MOMENT]);
            let mut out = Vec::new();
            for packet in search {
                let moment = packet.as_any().downcast_ref::<InputMoment>().unwrap();
                let line = format!("{:08X} {:04X}\r\n", moment.index, to_u16(&moment.inputs) ^ 0xFFFF);
                
                line.as_bytes().iter().for_each(|byte| out.push(*byte));
            }
            
            let result = std::fs::write(path.clone(), out);
            if result.is_err() { println!("Err: {:?}\n", result.err().unwrap()); return; }
            println!("Legacy file data has been exported to: {}\n", path.canonicalize().unwrap().to_string_lossy());
        },
        _ => ()
    }
}

fn cli_read(pretext: Option<&str>) -> Result<String, Option<Error>> {
    if pretext.is_some() {
        print!("{}", pretext.unwrap());
        flush();
    }
    
    let mut cli_input = String::new();
    let result = std::io::stdin().read_line(&mut cli_input);
    if result.is_err() {
        return Err(result.err());
    }
    println!("");
    
    Ok(cli_input.trim().to_string())
}

fn cli_selection<'a>(list: &[&str], pretext: Option<&str>, posttext: Option<&str>) -> usize {
    if pretext.is_some() {
        print!("{}", pretext.unwrap());
    }
    let padding = ((list.len() as f32).log10() as usize) + 1;
    for (i, element) in list.iter().enumerate() {
        println!("[{}]: {}", format!("{:padding$}", i, padding=padding).cyan(), element);
    }
    if posttext.is_some() {
        print!("{}", posttext.unwrap());
        flush();
    }
    
    let result = cli_read(None);
    if result.is_ok() {
        let text = result.unwrap();
        
        if !text.is_empty() {
            let result = text.parse::<usize>();
            if result.is_ok() {
                let selection = result.unwrap();
                if (0..list.len()).any(|i| i == selection) {
                    return selection;
                }
            }
        }
    }
    
    0
}

fn check_tasd_exists_create(path_ref: &mut PathBuf) {
    let mut path = path_ref.clone();
    if !path.extension().unwrap_or(OsStr::new("")).eq_ignore_ascii_case("tasd") { path = path.with_extension("tasd"); }
    if !path.exists() || !path.is_file() {
        let parent = path.parent();
        if parent.is_some() {
            std::fs::create_dir_all(parent.unwrap()).unwrap();
        }
        
        let result = std::fs::write(path.clone(), NEW_TASD_FILE);
        if result.is_err() {
            println!("Error: {:?}", result.err());
            exit(true, 1);
        }
        println!("Created new file: {}\n", path.to_string_lossy());
    } else {
        println!("Existing file found.\n");
    }
    flush();
    *path_ref = path;
}

fn exit(pause: bool, code: i32) {
    if pause {
        cli_read(Some("\nPress enter to exit...")).unwrap();
    }
    std::process::exit(code);
}

fn flush() {
    stdout().flush().expect("Flushing stdout failed. How did that happen?!");
}
