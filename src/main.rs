
mod util;
mod lookup;
mod definitions;
mod movie;

use std::ffi::OsStr;
use std::io::{Error, stdout, Write};
use std::path::{PathBuf, Path};

use crossterm::execute;
use crossterm::terminal::{SetTitle};
use regex::Regex;
use chrono::{NaiveDate, Date, NaiveTime, Utc};

use definitions::*;
use movie::TasdMovie;
use lookup::{key_spec_lut, console_type_lut, console_region_lut, memory_init_lut, transition_lut, controller_type_lut};
use util::{to_bytes, to_u16};
use std::cmp::max;


macro_rules! fstr {
    ($text:expr) => {
        String::from($text);
    };
}
macro_rules! some_fstr {
    ($text:expr) => {
        Some(String::from($text));
    };
}

fn main() {
    
    //execute!(stdout(), Clear(ClearType::All)).unwrap();
    execute!(stdout(), SetTitle("TASD-Edit")).unwrap();
    
    let mut tasd = None;
    
    let cli_state = parse_args();
    if cli_state.tasd_path.is_some() {
        tasd = Some(TasdMovie::new(&cli_state.tasd_path.unwrap()).unwrap());
    }
    
    
    
    //let start = Instant::now();
    //let end = Instant::now();
    //println!("Parse Time: {:.9} seconds\n", (end - start).as_secs_f64());
    
    while !main_menu(&mut tasd) {}
    
    exit(false, 0);
}

fn main_menu(tasd_option: &mut Option<TasdMovie>) -> bool {
    if tasd_option.is_some() {
        let tasd = tasd_option.as_mut().unwrap();
        let selection = cli_selection([
                fstr!("Exit/Quit"),
                fstr!("Add a new packet"),
                fstr!("Remove a packet"),
                fstr!("Display all packets"),
                fstr!("Display all, except input chunks"),
                fstr!("Save prettified packets to file"),
                fstr!("Create/load a TASD file"),
                fstr!("Import and append a legacy file"),
                fstr!("Export to legacy file"),
            ].iter(), Some(fstr!("What would you like to do?\n")), Some(fstr!("Option[0]: "))
        );
        
        let mut ret = false;
        match selection {
          //0 => exits program
            1 => { while !add_menu(tasd) {} },
            2 => { while !remove_menu(tasd) {} },
            3 => { display_packets(tasd, false); },
            4 => { display_packets(tasd, true); },
            5 => { save_pretty(tasd); },
            6 => { match load_tasd() {
                Err(x) => println!("Err: {}\n", x),
                Ok(x) => *tasd = x,
            }},
            7 => { match import_legacy(tasd_option) {
                Err(x) => println!("Err: {}\n", x),
                Ok(_) => ()
            }},
            8 => { export_legacy(tasd) },
            
            _ => ret = true,
        };
        
        ret
    } else {
        let selection = cli_selection([
                fstr!("Exit/Quit"),
                fstr!("Create/load a TASD file"),
                fstr!("Import a legacy file"),
            ].iter(), Some(fstr!("What would you like to do?\n")), Some(fstr!("Option[0]: "))
        );
        
        let mut ret = false;
        match selection {
          //0 => exits program
            1 => { match load_tasd() {
                Err(x) => println!("Err: {}\n", x),
                Ok(x) => *tasd_option = Some(x),
            }},
            2 => {
                match import_legacy(tasd_option) {
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
    let create = create_packet(some_fstr!("Select the packet you'd like to add.\n"), Some(vec![DUMP_LAST_MODIFIED]));
    
    if create.1.is_some() {
        let packet = create.1.unwrap();
        
        if packet.get_packet_spec().key != [0x00, 0x00] { // if packet is not unsupported...
            tasd.packets.push(packet);
            save_tasd(tasd);
            println!("New packet added to file!\n");
        }
    }
    
    create.0
}

fn create_packet(pretext: Option<String>, exclude: Option<Vec<[u8; 2]>>) -> (bool, Option<Box<dyn Packet>>){
    let exclude = exclude.unwrap_or(vec![]);
    let mut options = Vec::<String>::new();
    options.push(fstr!("Return to main menu"));
    let mut specs = Vec::new();
    for upper in 0..=255 {
        for lower in 0..=255 {
            let key = [upper, lower];
            if !exclude.contains(&key) {
                let spec = key_spec_lut(key);
                if spec.is_some() {
                    let spec = spec.unwrap();
                    specs.push(([upper, lower], spec.clone()));
                    options.push(format!("{} = {}", spec.name, spec.description));
                }
            }
        }
    }
    let selection = cli_selection(options.iter(), pretext, some_fstr!("Packet Type[0]: "));
    if selection == 0 { return (true, None); }
    
    let packet: Box<dyn Packet>;
    let spec = &specs[selection - 1];
    match spec.0 {
        CONSOLE_TYPE => {
            let mut options = Vec::new();
            let mut kinds = Vec::new();
            options.push(fstr!("Return to add menu"));
            for i in 1..=255 {
                let s = console_type_lut(i);
                if s.is_some() { options.push(fstr!(s.unwrap())); kinds.push(i); }
            }
            let selection = cli_selection(options.iter(), None, some_fstr!("Console Type[0]: "));
            if selection == 0 { return (false, None); }
            let kind = kinds[selection - 1];
            
            if kind == 0xFF {
                let text = cli_read(some_fstr!("Custom type: "));
                if text.is_err() { println!("Err: {:?}\n", text.err().unwrap()); return (false, None); }
                packet = ConsoleType::new(kind, Some(text.unwrap()));
            } else {
                packet = ConsoleType::new(kind, None);
            }
        },
        CONSOLE_REGION => {
            let mut options = Vec::new();
            let mut kinds = Vec::new();
            options.push(fstr!("Return to add menu"));
            for i in 1..=255 {
                let s = console_region_lut(i);
                if s.is_some() { options.push(fstr!(s.unwrap())); kinds.push(i); }
            }
            let selection = cli_selection(options.iter(), None, some_fstr!("Console Region[0]: "));
            if selection == 0 { return (false, None); }
            packet = ConsoleRegion::new(kinds[selection - 1]);
        },
        GAME_TITLE => {
            let text = cli_read(some_fstr!("Game title: "));
            if text.is_err() { println!("Err: {:?}\n", text.err().unwrap()); return (false, None); }
            packet = GameTitle::new(text.unwrap());
        },
        AUTHOR => {
            let text = cli_read(some_fstr!("Author: "));
            if text.is_err() { println!("Err: {:?}\n", text.err().unwrap()); return (false, None); }
            packet = Author::new(text.unwrap());
        },
        CATEGORY => {
            let text = cli_read(some_fstr!("Category: "));
            if text.is_err() { println!("Err: {:?}\n", text.err().unwrap()); return (false, None); }
            packet = Category::new(text.unwrap());
        },
        EMULATOR_NAME => {
            let text = cli_read(some_fstr!("Emulator name: "));
            if text.is_err() { println!("Err: {:?}\n", text.err().unwrap()); return (false, None); }
            packet = EmulatorName::new(text.unwrap());
        },
        EMULATOR_VERSION => {
            let text = cli_read(some_fstr!("Emulator version: "));
            if text.is_err() { println!("Err: {:?}\n", text.err().unwrap()); return (false, None); }
            packet = EmulatorVersion::new(text.unwrap());
        },
        EMULATOR_CORE => {
            let text = cli_read(some_fstr!("Emulator core: "));
            if text.is_err() { println!("Err: {:?}\n", text.err().unwrap()); return (false, None); }
            packet = EmulatorCore::new(text.unwrap());
        },
        TAS_LAST_MODIFIED => {
            let text = cli_read(some_fstr!("TAS last modified (epoch seconds, YYYY-MM-DD, or YYYY-MM-DD HH:MM:SS): "));
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
            packet = TASLastModified::new(epoch);
        },
        TOTAL_FRAMES => {
            let text = cli_read(some_fstr!("Total frames: "));
            if text.is_err() { println!("Err: {:?}\n", text.err().unwrap()); return (false, None); }
            let parse_attempt = text.unwrap().parse();
            if parse_attempt.is_err() { println!("Err: {:?}\n", parse_attempt.err().unwrap()); return (false, None); }
            packet = TotalFrames::new(parse_attempt.unwrap());
        },
        RERECORDS => {
            let text = cli_read(some_fstr!("Rerecord count: "));
            if text.is_err() { println!("Err: {:?}\n", text.err().unwrap()); return (false, None); }
            let parse_attempt = text.unwrap().parse();
            if parse_attempt.is_err() { println!("Err: {:?}\n", parse_attempt.err().unwrap()); return (false, None); }
            packet = Rerecords::new(parse_attempt.unwrap());
        },
        SOURCE_LINK => {
            let text = cli_read(some_fstr!("Source link/url: "));
            if text.is_err() { println!("Err: {:?}\n", text.err().unwrap()); return (false, None); }
            packet = SourceLink::new(text.unwrap());
        },
        BLANK_FRAMES => {
            let text = cli_read(some_fstr!("Blank frames (-32768 to +32767): "));
            if text.is_err() { println!("Err: {:?}\n", text.err().unwrap()); return (false, None); }
            let parse_attempt = text.unwrap().parse();
            if parse_attempt.is_err() { println!("Err: {:?}\n", parse_attempt.err().unwrap()); return (false, None); }
            packet = BlankFrames::new(parse_attempt.unwrap());
        },
        VERIFIED => {
            let text = cli_read(some_fstr!("Has been verified (true or false): "));
            if text.is_err() { println!("Err: {:?}\n", text.err().unwrap()); return (false, None); }
            let parse_attempt = text.unwrap().parse::<bool>();
            if parse_attempt.is_err() { println!("Err: {:?}\n", parse_attempt.err().unwrap()); return (false, None); }
            packet = Verified::new(parse_attempt.unwrap() as u8);
        },
        MEMORY_INIT => {
            let mut options = Vec::new();
            let mut kinds = Vec::new();
            options.push(fstr!("Return to add menu"));
            for i in 1..=255 {
                let s = memory_init_lut(i);
                if s.is_some() { options.push(fstr!(s.unwrap())); kinds.push(i); }
            }
            let selection = cli_selection(options.iter(), None, some_fstr!("Initialization type[0]: "));
            if selection == 0 { return (false, None); }
            let kind = kinds[selection - 1];
            
            let required = cli_read(some_fstr!("Required for verification (true or false): "));
            if required.is_err() { println!("Err: {:?}\n", required.err().unwrap()); return (false, None); }
            let required = required.unwrap().parse::<bool>();
            if required.is_err() { println!("Err: {:?}\n", required.err().unwrap()); return (false, None); }
            
            let name = cli_read(some_fstr!("Name of memory space: "));
            if name.is_err() { println!("Err: {:?}\n", name.err().unwrap()); return (false, None); }
            let mut payload = None;
            if kind == 0x02 {
                let path = cli_read(some_fstr!("Path to file containing memory: "));
                if path.is_err() { println!("Err: {:?}\n", path.err().unwrap()); return (false, None); }
                let path = path.unwrap();
                let path = Path::new(&path);
                if !path.exists() || !path.is_file() { println!("Path either doesn't exist or isn't a file.\n"); return (false, None); }
                let data_result = std::fs::read(path);
                if data_result.is_err() { println!("Err: {:?}\n", data_result.err().unwrap()); return (false, None); }
                payload = Some(data_result.unwrap());
            }
            packet = MemoryInit::new(kind, required.unwrap() as u8, name.unwrap(), payload);
        },
        PORT_CONTROLLER => {
            let text = cli_read(some_fstr!("Port number (1-indexed): "));
            if text.is_err() { println!("Err: {:?}\n", text.err().unwrap()); return (false, None); }
            let parse_attempt = text.unwrap().parse();
            if parse_attempt.is_err() { println!("Err: {:?}\n", parse_attempt.err().unwrap()); return (false, None); }
            let port = parse_attempt.unwrap();
            
            let mut options = Vec::new();
            let mut kinds = Vec::new();
            options.push(fstr!("Return to add menu"));
            for i in 1..=0xFFFF {
                let s = controller_type_lut(i);
                if s.is_some() { options.push(fstr!(s.unwrap())); kinds.push(i); }
            }
            let selection = cli_selection(options.iter(), None, some_fstr!("Initialization type[0]: "));
            if selection == 0 { return (false, None); }
            let kind = kinds[selection - 1];
            
            packet = PortController::new(port, kind);
        }
        
        LATCH_FILTER => {
            let text = cli_read(some_fstr!("Latch filter (specify positive integer; which will be multiplied by 0.1ms): "));
            if text.is_err() { println!("Err: {:?}\n", text.err().unwrap()); return (false, None); }
            let parse_attempt = text.unwrap().parse();
            if parse_attempt.is_err() { println!("Err: {:?}\n", parse_attempt.err().unwrap()); return (false, None); }
            packet = LatchFilter::new(parse_attempt.unwrap());
        },
        CLOCK_FILTER => {
            let text = cli_read(some_fstr!("Clock filter (specify positive integer; which will be multiplied by 0.25us): "));
            if text.is_err() { println!("Err: {:?}\n", text.err().unwrap()); return (false, None); }
            let parse_attempt = text.unwrap().parse();
            if parse_attempt.is_err() { println!("Err: {:?}\n", parse_attempt.err().unwrap()); return (false, None); }
            packet = ClockFilter::new(parse_attempt.unwrap());
        },
        OVERREAD => {
            let text = cli_read(some_fstr!("Overread (0 for a HIGH signal, or 1 for LOW): "));
            if text.is_err() { println!("Err: {:?}\n", text.err().unwrap()); return (false, None); }
            let parse_attempt = text.unwrap().parse();
            if parse_attempt.is_err() { println!("Err: {:?}\n", parse_attempt.err().unwrap()); return (false, None); }
            packet = Overread::new(parse_attempt.unwrap());
        },
        GAME_GENIE_CODE => {
            let text = cli_read(some_fstr!("Game genie code (6 or 8 characters): "));
            if text.is_err() { println!("Err: {:?}\n", text.err().unwrap()); return (false, None); }
            packet = GameGenieCode::new(text.unwrap());
        },
        
        //TODO: INPUT_CHUNKS: Idea is to list all frames with an index, and let user specify before or after a specific index to insert a new packet.
        //TODO: INPUT_MOMENT: Much easier to support
        
        TRANSITION => {
            let index = cli_read(some_fstr!("Frame/Index number: "));
            if index.is_err() { println!("Err: {:?}\n", index.err().unwrap()); return (false, None); }
            let parse_attempt = index.unwrap().parse::<u32>();
            if parse_attempt.is_err() { println!("Err: {:?}\n", parse_attempt.err().unwrap()); return (false, None); }
            
            let mut options = Vec::new();
            let mut kinds = Vec::new();
            options.push(fstr!("Return to add menu"));
            for i in 1..=255 {
                let s = transition_lut(i);
                if s.is_some() { options.push(fstr!(s.unwrap())); kinds.push(i); }
            }
            let selection = cli_selection(options.iter(), None, some_fstr!("Transition type[0]: "));
            if selection == 0 { return (false, None); }
            let kind = kinds[selection - 1];
            
            let mut payload = None;
            if kind == 0xFF {
                let create = create_packet(some_fstr!("Select a packet for this transition.\n"), Some(vec![DUMP_LAST_MODIFIED, INPUT_CHUNKS, TRANSITION, LAG_FRAME_CHUNK, MOVIE_TRANSITION]));
                if create.0 || create.1.is_none() { return (true, None); }
                payload = Some(create.1.unwrap().get_raw());
            }
            packet = Transition::new(parse_attempt.unwrap(), kind, payload);
        },
        LAG_FRAME_CHUNK => {
            let index = cli_read(some_fstr!("Frame/Index number: "));
            if index.is_err() { println!("Err: {:?}\n", index.err().unwrap()); return (false, None); }
            let index = index.unwrap().parse::<u32>();
            if index.is_err() { println!("Err: {:?}\n", index.err().unwrap()); return (false, None); }
            
            let length = cli_read(some_fstr!("Length of chunk: "));
            if length.is_err() { println!("Err: {:?}\n", length.err().unwrap()); return (false, None); }
            let length = length.unwrap().parse::<u32>();
            if length.is_err() { println!("Err: {:?}\n", length.err().unwrap()); return (false, None); }
            packet = LagFrameChunk::new(index.unwrap(), length.unwrap());
        },
        MOVIE_TRANSITION => {
            let index = cli_read(some_fstr!("Frame/Index number: "));
            if index.is_err() { println!("Err: {:?}\n", index.err().unwrap()); return (false, None); }
            let parse_attempt = index.unwrap().parse::<u32>();
            if parse_attempt.is_err() { println!("Err: {:?}\n", parse_attempt.err().unwrap()); return (false, None); }
            
            let mut options = Vec::new();
            let mut kinds = Vec::new();
            options.push(fstr!("Return to add menu"));
            for i in 1..=255 {
                let s = transition_lut(i);
                if s.is_some() { options.push(fstr!(s.unwrap())); kinds.push(i); }
            }
            let selection = cli_selection(options.iter(), None, some_fstr!("Transition type[0]: "));
            if selection == 0 { return (false, None); }
            let kind = kinds[selection - 1];
            
            let mut payload = None;
            if kind == 0xFF {
                let create = create_packet(some_fstr!("Select a packet for this transition.\n"), Some(vec![DUMP_LAST_MODIFIED, INPUT_CHUNKS, TRANSITION, LAG_FRAME_CHUNK, MOVIE_TRANSITION]));
                if create.0 || create.1.is_none() { return (true, None); }
                payload = Some(create.1.unwrap().get_raw());
            }
            packet = MovieTransition::new(parse_attempt.unwrap(), kind, payload);
        },
        _ => { println!("Sorry, creating new packets of this type is currently unsupported.\n"); return (false, None); },
    }
    
    (false, Some(packet))
}

fn remove_menu(tasd: &mut TasdMovie) -> bool {
    let mut options = Vec::<String>::new();
    options.push(fstr!("Return to main menu"));
    for packet in &tasd.packets {
        options.push(format!("{}: {}", packet.get_packet_spec().name, packet.formatted_payload()));
    }
    
    let selection = cli_selection(options.iter(), Some(fstr!("Select the packet you wish to remove.\n")), Some(fstr!("Packet index[0]: ")));
    if selection != 0 {
        println!("Packet removed.\n");
        tasd.packets.remove(selection - 1);
        save_tasd(tasd);
        return false;
    }
    
    true
}

fn display_packets(tasd: &TasdMovie, exclude_inputs: bool) {
    let pretty = prettify_packets(tasd);
    for packet in pretty {
        if exclude_inputs && packet.contains("InputChunks:") { continue; }
        println!("{}", packet);
    }
    println!("");
}

fn save_pretty(tasd: &TasdMovie) {
    let pretty = prettify_packets(tasd);
    let mut path = tasd.source_file.clone();
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
    
    out.push(format!("Version: {:#06X}, Key Width: {}", tasd.version, tasd.key_width));
    let padding = ((tasd.packets.len() as f32).log10() as usize) + 1;
    for (i, packet) in tasd.packets.iter().enumerate() {
        out.push(format!("[{:padding$.0}]: {}: {}", i + 1, packet.get_packet_spec().name, packet.formatted_payload(), padding=padding));
    }
    
    out
}

fn save_tasd(tasd: &mut TasdMovie) {
    let mut exists = false;
    let mut removals = Vec::new();
    for (i, packet) in tasd.packets.iter_mut().enumerate() {
        match packet.get_packet_spec().key {
            DUMP_LAST_MODIFIED => {
                if exists {
                    removals.push(i);
                } else {
                    let epoch = Utc::now().timestamp();
                    *packet = DumpLastModified::new(epoch);
                    exists = true;
                }
            },
            _ => ()
        }
    }
    removals.iter().for_each(|i| { tasd.packets.remove(*i); });
    
    if !exists {
        let epoch = Utc::now().timestamp();
        let packet = DumpLastModified::new(epoch);
        tasd.packets.push(packet);
    }
    
    tasd.save();
}

#[derive(Default)]
struct CliState {
    tasd_path: Option<PathBuf>,
    
}

fn parse_args() -> CliState {
    let args: Vec<String> = std::env::args().collect();
    
    let single_dash_regex = Regex::new("^-[^-]").unwrap();
    let double_dash_regex = Regex::new("^--[^-]").unwrap();
    let     no_dash_regex = Regex::new("^[^-]").unwrap();
    
    let _single_dash_args: Vec<String> = args.iter().filter(|arg| single_dash_regex.is_match(arg)).map(|arg| arg.clone()).collect();
    let _double_dash_args: Vec<String> = args.iter().filter(|arg| double_dash_regex.is_match(arg)).map(|arg| arg.clone()).collect();
    let text_args: Vec<String> =        args.iter().filter(|arg|     no_dash_regex.is_match(arg)).map(|arg| arg.clone()).collect();
    
    let mut state = CliState::default();
    
    let mut path_args = Vec::<PathBuf>::new();
    for arg in text_args.clone() {
        let path = PathBuf::from(arg.clone());
        
        if !arg.eq(&text_args[0]) {
            path_args.push(path);
        }
        
        /*let path_ext = path.with_extension("tasd");
        
        if path.exists() && path.is_file() {
            let result = std::fs::read(path.clone());
            if result.is_ok() && result.unwrap()[0..4].eq(spec::MAGIC_NUMBER) {
                path_args.push(path);
            }
        } else if path_ext.exists() && path_ext.is_file() {
            let result = std::fs::read(path_ext.clone());
            if result.is_ok() && result.unwrap()[0..4].eq(spec::MAGIC_NUMBER) {
                path_args.push(path_ext);
            }
        } else {
            path_args.push(path);
        }*/
    }
    
    if path_args.len() > 1 {
        // More than one valid path was provided. Request the user to select which one they want to load. //
        
        let i = cli_selection(
            path_args.iter().map(|path| path.to_string_lossy()),
            Some("Looks like you provided multiple paths to possible tasd files.\n".to_string()),
            Some("Please select which you want to create/load [0]: ".to_string())
        );
        let mut path = path_args[i].clone();
        check_tasd_exists_create(&mut path);
        
        state.tasd_path = Some(path);
    } else if path_args.len() == 1 {
        // Only one valid path was provided, using it. //
        
        let mut path = path_args[0].clone();
        check_tasd_exists_create(&mut path);
        
        state.tasd_path = Some(path);
    }
    
    //TODO implement additional argument functionality
    
    
    
    state
}

fn load_tasd() -> Result<TasdMovie, String> {
    let result = cli_read(some_fstr!("Provide the name for a new empty file, or the path to an existing file you wish to load.\nFile name: "));
    if result.is_ok() {
        let mut name = result.unwrap();
        if name.is_empty() {
            return Err(fstr!("Err: Empty input. You must create or load a file to use this software."));
        } else {
            if !name.ends_with(".tasd") { name.push_str(".tasd") }
            let mut path = PathBuf::from(name);
            check_tasd_exists_create(&mut path);
            let result = TasdMovie::new(&path);
            if result.is_err() {
                return Err(result.err().unwrap_or(fstr!("Unknown error")));
            } else {
                return Ok(result.unwrap());
            }
        }
    } else {
        return Err(format!("Err: {:?}", result.err().unwrap()));
    }
}

fn import_legacy(tasd_option: &mut Option<TasdMovie>) -> Result<(), String> {
    let result = cli_read(some_fstr!("Path to .r08, .r16m, or GBI(.txt) file: "));
    if result.is_err() { return Err(format!("Err: {:?}", result.err())) }
    let path = PathBuf::from(result.unwrap());
    if !path.exists() || path.is_dir() { return Err(fstr!("File either doesn't exist or is a directory.")) }
    if path.extension().is_none() { return Err(fstr!("Unable to identify file, make sure extension is correct."))}
    let extension = path.extension().unwrap().to_string_lossy().to_string();
    
    if tasd_option.is_none() {
        let mut tasd = TasdMovie::default();
        tasd.source_file = path.clone();
        tasd.source_file.set_extension("tasd");
        tasd_option.insert(tasd);
    }
    let tasd = tasd_option.as_mut().unwrap();
    
    match extension.as_str() {
        "r08" => {
            let result = std::fs::read(path);
            if result.is_err() { return Err(format!("Err: {:?}", result.err())) }
            let mut result = result.unwrap();
            if result.len() % 2 == 1 { result.push(0xFF) } // Shouldn't ever be misaligned, but is safer to double check
            tasd.packets.push(ConsoleType::new(0x01, None));
            tasd.packets.push(PortController::new(1, 0x0101));
            tasd.packets.push(PortController::new(2, 0x0101));
            
            let mut port1 = Vec::new();
            let mut port2 = Vec::new();
            for i in 0..(result.len() / 2) {
                port1.push(result[(i * 2)] ^ 0xFF);
                port2.push(result[(i * 2) + 1] ^ 0xFF);
            }
            tasd.packets.push(InputChunks::new(1, port1));
            tasd.packets.push(InputChunks::new(2, port2));
            
            save_tasd(tasd);
            println!("Legacy file data has been imported and saved.\n");
            Ok(())
        },
        "r16m" => {
            let result = std::fs::read(path);
            if result.is_err() { return Err(format!("Err: {:?}", result.err())) }
            let mut result = result.unwrap();
            if result.len() % 2 == 1 { result.push(0) } // Shouldn't ever be misaligned, but is safer to double check
            if result.len() % 4 == 1 { result.push(0); result.push(0); } // Shouldn't ever be misaligned, but is safer to double check
            tasd.packets.push(ConsoleType::new(0x02, None));
            tasd.packets.push(PortController::new(1, 0x0201));
            tasd.packets.push(PortController::new(2, 0x0201));
            
            let mut port1 = Vec::new();
            let mut port2 = Vec::new();
            for i in 0..(result.len() / 4) {
                port1.push(result[(i * 4)] ^ 0xFF);
                port1.push(result[(i * 4) + 1] ^ 0xFF);
                port2.push(result[(i * 4) + 2] ^ 0xFF);
                port2.push(result[(i * 4) + 3] ^ 0xFF);
            }
            tasd.packets.push(InputChunks::new(1, port1));
            tasd.packets.push(InputChunks::new(2, port2));
            
            save_tasd(tasd);
            println!("Legacy file data has been imported and saved.\n");
            Ok(())
        },
        "txt" => {
            let selection = cli_selection([fstr!("GB"), fstr!("GBC"), fstr!("GBA")].iter(), some_fstr!("Which handheld is this for?\n"), some_fstr!("Handheld type[0]: "));
            let result = std::fs::read_to_string(path);
            if result.is_err() { return Err(format!("Err: {:?}", result.err())) }
            let result = result.unwrap();
            let result = result.lines();
            match selection {
                0 => {
                    tasd.packets.push(ConsoleType::new(0x05, None));
                    tasd.packets.push(PortController::new(1, 0x0501));
                },
                1 => {
                    tasd.packets.push(ConsoleType::new(0x06, None));
                    tasd.packets.push(PortController::new(1, 0x0601));
                },
                2 => {
                    tasd.packets.push(ConsoleType::new(0x07, None));
                    tasd.packets.push(PortController::new(1, 0x0701));
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
                        0 => { tasd.packets.push(InputMoment::new(1, clock, vec![input[1] ^ 0xFF])) },
                        1 => { tasd.packets.push(InputMoment::new(1, clock, vec![input[1] ^ 0xFF])) },
                        2 => { tasd.packets.push(InputMoment::new(1, clock, vec![input[0] ^ 0xFF, input[1] ^ 0xFF])) },
                        _ => ()
                    }
                }
            }
            
            save_tasd(tasd);
            println!("Legacy file data has been imported and saved.\n");
            Ok(())
        },
        _ => Err(fstr!("Unable to identify file, make sure extension is correct."))
    }
}

fn export_legacy(tasd: &TasdMovie) {
    let valid_types = [0x01, 0x02, 0x05, 0x06, 0x07];
    let search = tasd.search_by_key(vec![CONSOLE_TYPE]);
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
            let mut options = vec![fstr!("Return to main menu")];
            for packet in &packets {
                if let Some(kind) = console_type_lut(packet.kind) {
                    options.push(fstr!(kind));
                }
            }
            let selection = cli_selection(options.iter(), some_fstr!("Multiple console types detected. Select which you're trying to export to."), some_fstr!("Console type[0]: "));
            if selection == 0 { return; }
            
            packets[selection - 1].kind 
        },
    };
    
    match console_type {
        0x01 => { // NES (.r08)
            let path = tasd.source_file.with_extension("export.r08");
            let search = tasd.search_by_key(vec![INPUT_CHUNKS]);
            
            let mut port1 = Vec::new();
            let mut port2 = Vec::new();
            for packet in search {
                let chunk = packet.as_any().downcast_ref::<InputChunks>().unwrap();
                if chunk.port == 1 { chunk.payload.iter().for_each(|byte| port1.push(*byte ^ 0xFF)) }
                if chunk.port == 2 { chunk.payload.iter().for_each(|byte| port2.push(*byte ^ 0xFF)) }
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
            let path = tasd.source_file.with_extension("export.r16m");
            let search = tasd.search_by_key(vec![INPUT_CHUNKS]);
            
            let mut port1 = Vec::new();
            let mut port2 = Vec::new();
            let mut port3 = Vec::new();
            let mut port4 = Vec::new();
            for packet in search {
                let chunk = packet.as_any().downcast_ref::<InputChunks>().unwrap();
                if chunk.port == 1 { chunk.payload.iter().for_each(|byte| port1.push(*byte ^ 0xFF)) }
                if chunk.port == 2 { chunk.payload.iter().for_each(|byte| port2.push(*byte ^ 0xFF)) }
                if chunk.port == 3 { chunk.payload.iter().for_each(|byte| port3.push(*byte ^ 0xFF)) }
                if chunk.port == 4 { chunk.payload.iter().for_each(|byte| port4.push(*byte ^ 0xFF)) }
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
            let path = tasd.source_file.with_extension("export.txt");
            let search = tasd.search_by_key(vec![INPUT_MOMENT]);
            let mut out = Vec::new();
            for packet in search {
                let moment = packet.as_any().downcast_ref::<InputMoment>().unwrap();
                let line = format!("{:08X} {:04X}\n", moment.index, (moment.payload[0] ^ 0xFF) as u16);
                
                line.as_bytes().iter().for_each(|byte| out.push(*byte));
            }
            
            let result = std::fs::write(path.clone(), out);
            if result.is_err() { println!("Err: {:?}\n", result.err().unwrap()); return; }
            println!("Legacy file data has been exported to: {}\n", path.canonicalize().unwrap().to_string_lossy());
        },
        0x07 => { // GBA (GBI .txt)
            let path = tasd.source_file.with_extension("export.txt");
            let search = tasd.search_by_key(vec![INPUT_MOMENT]);
            let mut out = Vec::new();
            for packet in search {
                let moment = packet.as_any().downcast_ref::<InputMoment>().unwrap();
                let line = format!("{:08X} {:04X}\n", moment.index, to_u16(&moment.payload) ^ 0xFFFF);
                
                line.as_bytes().iter().for_each(|byte| out.push(*byte));
            }
            
            let result = std::fs::write(path.clone(), out);
            if result.is_err() { println!("Err: {:?}\n", result.err().unwrap()); return; }
            println!("Legacy file data has been exported to: {}\n", path.canonicalize().unwrap().to_string_lossy());
        },
        _ => ()
    }
}

fn cli_read(pretext: Option<String>) -> Result<String, Option<Error>> {
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

fn cli_selection<S: Into<String>, I: Iterator<Item=S>>(list: I, pretext: Option<String>, posttext: Option<String>) -> usize {
    let list: Vec<String> = list.map(|element| element.into()).collect();
    
    if pretext.is_some() {
        print!("{}", pretext.unwrap());
    }
    let padding = ((list.len() as f32).log10() as usize) + 1;
    for (i, element) in list.iter().enumerate() {
        println!("[{:padding$.0}]: {}", i, element, padding=padding);
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
        println!("Created new file: {}", path.to_string_lossy());
    } else {
        println!("Existing file found.");
    }
    flush();
    *path_ref = path;
}

fn exit(pause: bool, code: i32) {
    if pause {
        cli_read(Some("\nPress enter to exit...".to_string())).unwrap();
    }
    std::process::exit(code);
}

fn flush() {
    stdout().flush().expect("Flushing stdout failed. How did that happen?!");
}
