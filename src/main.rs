use std::ffi::OsStr;
use std::io::{Error, stdout, Write};
use std::path::{PathBuf, Path};
use std::time::Instant;

use crossterm::execute;
use crossterm::terminal::{SetTitle};
use regex::Regex;

use definitions::*;
use movie::TasdMovie;
use crate::lookup::{key_spec_lut, console_type_lut, console_region_lut, memory_init_lut, transition_lut};
use chrono::{NaiveDate, Date, NaiveTime, Utc};

mod util;
mod lookup;
mod definitions;
mod movie;

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
    
    let cli_state = parse_args();
    let start = Instant::now();
    let mut tasd = TasdMovie::new(&cli_state.tasd_path).unwrap();
    let end = Instant::now();
    println!("Parse Time: {:.9} seconds\n", (end - start).as_secs_f64());
    
    while !main_menu(&mut tasd) {}
    
    exit(false, 0);
}

fn main_menu(tasd: &mut TasdMovie) -> bool {
    let selection = cli_selection(
        [
            fstr!("Exit/Quit"),
            fstr!("Modify a packet (unimplemented)"),
            fstr!("Add a new packet"),
            fstr!("Remove a packet"),
            fstr!("Display all packets"),
        ].iter(), Some(fstr!("What would you like to do?\n")), Some(fstr!("Option[0]: "))
    );
    
    match selection {
        1 => {
            edit_menu(tasd);
            false
        },
        2 => {
            while !add_menu(tasd) {}
            false
        },
        3 => {
            while !remove_menu(tasd) {}
            false
        },
        4 => {
            display_packets(tasd);
            false
        },
        
        _ => true,
    }
}

#[allow(unused)]
fn edit_menu(tasd: &mut TasdMovie) {
    
}

fn add_menu(tasd: &mut TasdMovie) -> bool {
    let mut options = Vec::<String>::new();
    options.push(fstr!("Return to main menu"));
    let mut specs = Vec::new();
    for upper in 0..=255 {
        for lower in 0..=255 {
            let key = [upper, lower];
            if key != DUMP_LAST_MODIFIED {
                let spec = key_spec_lut(key);
                if spec.is_some() {
                    let spec = spec.unwrap();
                    specs.push(([upper, lower], spec.clone()));
                    options.push(format!("{} = {}", spec.name, spec.description));
                }
            }
        }
    }
    let selection = cli_selection(options.iter(), some_fstr!("Select the packet you'd like to add.\n"), some_fstr!("Packet Type[0]: "));
    if selection == 0 { return true; }
    
    let spec = &specs[selection - 1];
    let packet: Box<dyn Packet>;
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
            if selection == 0 { return false; }
            packet = ConsoleType::new(kinds[selection - 1]);
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
            if selection == 0 { return false; }
            packet = ConsoleRegion::new(kinds[selection - 1]);
        },
        GAME_TITLE => {
            let text = cli_read(some_fstr!("Game title: "));
            if text.is_err() { println!("Err: {:?}\n", text.err().unwrap()); return false; }
            packet = GameTitle::new(text.unwrap());
        },
        AUTHOR => {
            let text = cli_read(some_fstr!("Author: "));
            if text.is_err() { println!("Err: {:?}\n", text.err().unwrap()); return false; }
            packet = Author::new(text.unwrap());
        },
        CATEGORY => {
            let text = cli_read(some_fstr!("Category: "));
            if text.is_err() { println!("Err: {:?}\n", text.err().unwrap()); return false; }
            packet = Category::new(text.unwrap());
        },
        EMULATOR_NAME => {
            let text = cli_read(some_fstr!("Emulator name: "));
            if text.is_err() { println!("Err: {:?}\n", text.err().unwrap()); return false; }
            packet = EmulatorName::new(text.unwrap());
        },
        EMULATOR_VERSION => {
            let text = cli_read(some_fstr!("Emulator version: "));
            if text.is_err() { println!("Err: {:?}\n", text.err().unwrap()); return false; }
            packet = EmulatorVersion::new(text.unwrap());
        },
        TAS_LAST_MODIFIED => {
            let text = cli_read(some_fstr!("TAS last modified (epoch seconds, YYYY-MM-DD, or YYYY-MM-DD HH:MM:SS): "));
            if text.is_err() { println!("Err: {:?}\n", text.err().unwrap()); return false; }
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
                if parse_attempt.is_err() { println!("Err: {:?}\n", parse_attempt.err().unwrap()); return false; }
                epoch = parse_attempt.unwrap();
            }
            packet = TASLastModified::new(epoch);
        },
        TOTAL_FRAMES => {
            let text = cli_read(some_fstr!("Total frames: "));
            if text.is_err() { println!("Err: {:?}\n", text.err().unwrap()); return false; }
            let parse_attempt = text.unwrap().parse();
            if parse_attempt.is_err() { println!("Err: {:?}\n", parse_attempt.err().unwrap()); return false; }
            packet = TotalFrames::new(parse_attempt.unwrap());
        },
        RERECORDS => {
            let text = cli_read(some_fstr!("Rerecord count: "));
            if text.is_err() { println!("Err: {:?}\n", text.err().unwrap()); return false; }
            let parse_attempt = text.unwrap().parse();
            if parse_attempt.is_err() { println!("Err: {:?}\n", parse_attempt.err().unwrap()); return false; }
            packet = Rerecords::new(parse_attempt.unwrap());
        },
        SOURCE_LINK => {
            let text = cli_read(some_fstr!("Source link/url: "));
            if text.is_err() { println!("Err: {:?}\n", text.err().unwrap()); return false; }
            packet = SourceLink::new(text.unwrap());
        },
        BLANK_FRAMES => {
            let text = cli_read(some_fstr!("Blank frames (-32768 to +32767): "));
            if text.is_err() { println!("Err: {:?}\n", text.err().unwrap()); return false; }
            let parse_attempt = text.unwrap().parse();
            if parse_attempt.is_err() { println!("Err: {:?}\n", parse_attempt.err().unwrap()); return false; }
            packet = BlankFrames::new(parse_attempt.unwrap());
        },
        VERIFIED => {
            let text = cli_read(some_fstr!("Has been verified (true or false): "));
            if text.is_err() { println!("Err: {:?}\n", text.err().unwrap()); return false; }
            let parse_attempt = text.unwrap().parse::<bool>();
            if parse_attempt.is_err() { println!("Err: {:?}\n", parse_attempt.err().unwrap()); return false; }
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
            if selection == 0 { return false; }
            let kind = kinds[selection - 1];
            
            let name = cli_read(some_fstr!("Name of memory space: "));
            if name.is_err() { println!("Err: {:?}\n", name.err().unwrap()); return false; }
            let mut payload = None;
            if kind == 0x02 {
                let path = cli_read(some_fstr!("Path to file containing memory: "));
                if path.is_err() { println!("Err: {:?}\n", path.err().unwrap()); return false; }
                let path = path.unwrap();
                let path = Path::new(&path);
                if !path.exists() || !path.is_file() { println!("Path either doesn't exist or isn't a file.\n"); return false; }
                let data_result = std::fs::read(path);
                if data_result.is_err() { println!("Err: {:?}\n", data_result.err().unwrap()); return false; }
                payload = Some(data_result.unwrap());
            }
            packet = MemoryInit::new(kind, name.unwrap(), payload);
        },
        
        LATCH_FILTER => {
            let text = cli_read(some_fstr!("Latch filter (specify positive integer; which will be multiplied by 0.1ms): "));
            if text.is_err() { println!("Err: {:?}\n", text.err().unwrap()); return false; }
            let parse_attempt = text.unwrap().parse();
            if parse_attempt.is_err() { println!("Err: {:?}\n", parse_attempt.err().unwrap()); return false; }
            packet = LatchFilter::new(parse_attempt.unwrap());
        },
        CLOCK_FILTER => {
            let text = cli_read(some_fstr!("Clock filter (specify positive integer; which will be multiplied by 0.25us): "));
            if text.is_err() { println!("Err: {:?}\n", text.err().unwrap()); return false; }
            let parse_attempt = text.unwrap().parse();
            if parse_attempt.is_err() { println!("Err: {:?}\n", parse_attempt.err().unwrap()); return false; }
            packet = ClockFilter::new(parse_attempt.unwrap());
        },
        OVERREAD => {
            let text = cli_read(some_fstr!("Overread (0 for a HIGH signal, or 1 for LOW): "));
            if text.is_err() { println!("Err: {:?}\n", text.err().unwrap()); return false; }
            let parse_attempt = text.unwrap().parse();
            if parse_attempt.is_err() { println!("Err: {:?}\n", parse_attempt.err().unwrap()); return false; }
            packet = Overread::new(parse_attempt.unwrap());
        },
        DPCM => {
            let text = cli_read(some_fstr!("Game is affected by DPCM (true or false): "));
            if text.is_err() { println!("Err: {:?}\n", text.err().unwrap()); return false; }
            let parse_attempt = text.unwrap().parse::<bool>();
            if parse_attempt.is_err() { println!("Err: {:?}\n", parse_attempt.err().unwrap()); return false; }
            packet = Dpcm::new(parse_attempt.unwrap() as u8);
        },
        GAME_GENIE_CODE => {
            let text = cli_read(some_fstr!("Game genie code (6 or 8 characters): "));
            if text.is_err() { println!("Err: {:?}\n", text.err().unwrap()); return false; }
            packet = GameGenieCode::new(text.unwrap());
        },
        
        //TODO: INPUT_CHUNKS: Idea is to list all frames with an index, and let user specify before or after a specific index to insert a new packet.
        
        TRANSITION => {
            let index = cli_read(some_fstr!("Frame/Index number: "));
            if index.is_err() { println!("Err: {:?}\n", index.err().unwrap()); return false; }
            let parse_attempt = index.unwrap().parse::<u32>();
            if parse_attempt.is_err() { println!("Err: {:?}\n", parse_attempt.err().unwrap()); return false; }
            
            let mut options = Vec::new();
            let mut kinds = Vec::new();
            options.push(fstr!("Return to add menu"));
            for i in 1..=255 {
                let s = transition_lut(i);
                if s.is_some() { options.push(fstr!(s.unwrap())); kinds.push(i); }
            }
            let selection = cli_selection(options.iter(), None, some_fstr!("Transition type[0]: "));
            if selection == 0 { return false; }
            let kind = kinds[selection - 1];
            
            let mut payload = None;
            if kind == 0x03 {
                unimplemented!();
                //TODO: Controller Swap transition must wait for decision about how to identify controllers.
            }
            packet = Transition::new(parse_attempt.unwrap(), kind, payload);
        },
        LAG_FRAME_CHUNK => {
            let index = cli_read(some_fstr!("Frame/Index number: "));
            if index.is_err() { println!("Err: {:?}\n", index.err().unwrap()); return false; }
            let index = index.unwrap().parse::<u32>();
            if index.is_err() { println!("Err: {:?}\n", index.err().unwrap()); return false; }
            
            let length = cli_read(some_fstr!("Length of chunk: "));
            if length.is_err() { println!("Err: {:?}\n", length.err().unwrap()); return false; }
            let length = length.unwrap().parse::<u32>();
            if length.is_err() { println!("Err: {:?}\n", length.err().unwrap()); return false; }
            packet = LagFrameChunk::new(index.unwrap(), length.unwrap());
        },
        _ => { println!("Sorry, creating new packets of this type is currently unsupported.\n"); return false; }
    }
    
    if packet.get_packet_spec().key != [0x00, 0x00] { // if packet is not unsupported...
        tasd.packets.push(packet);
        save_tasd(tasd);
        println!("New packet added to file!\n");
    }
    
    false
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

fn display_packets(tasd: &TasdMovie) {
    println!("Version: {:#06X}, Key Width: {}", tasd.version, tasd.key_width);
    let padding = ((tasd.packets.len() as f32).log10() as usize) + 1;
    for (i, packet) in tasd.packets.iter().enumerate() {
        println!("[{:padding$.0}]: {}: {}", i + 1, packet.get_packet_spec().name, packet.formatted_payload(), padding=padding);
    }
    println!();
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
    tasd_path: PathBuf,
    
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
    
    if path_args.is_empty() {
        // No file specified, request the user to input a path. //
        
        let result = cli_read(Some("No input tasd file was provided/found.\nProvide the name for a new empty file, or the path to an existing file you wish to load.\nFile name: ".to_string()));
        if result.is_ok() {
            let mut name = result.unwrap();
            if name.is_empty() {
                println!("Error: Empty input. You must create or load a file to use this software.");
                exit(true, 1);
            } else {
                if !name.ends_with(".tasd") { name.push_str(".tasd") }
                let mut path = PathBuf::from(name);
                check_tasd_exists_create(&mut path);
                
                state.tasd_path = path;
            }
        } else {
            println!("Error: {:?}", result.err().unwrap());
            exit(true, 1);
        }
    } else if path_args.len() > 1 {
        // More than one valid path was provided. Request the user to select which one they want to load. //
        
        let i = cli_selection(
            path_args.iter().map(|path| path.to_string_lossy()),
            Some("Looks like you provided multiple paths to possible tasd files.\n".to_string()),
            Some("Please select which you want to create/load [0]: ".to_string())
        );
        let mut path = path_args[i].clone();
        check_tasd_exists_create(&mut path);
        
        state.tasd_path = path;
    } else {
        // Only one valid path was provided, using it. //
        
        let mut path = path_args[0].clone();
        check_tasd_exists_create(&mut path);
        
        state.tasd_path = path;
    }
    
    //TODO implement additional argument functionality
    
    
    
    state
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
