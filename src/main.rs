use std::ffi::OsStr;
use std::io::{Error, stdout, Write};
use std::path::PathBuf;
use std::time::Instant;

use crossterm::execute;
use crossterm::terminal::{SetTitle};
use regex::Regex;

use definitions::NEW_TASD_FILE;
use movie::TasdMovie;

mod util;
mod lookup;
mod definitions;
mod movie;

macro_rules! fstr {
    ($text:expr) => {
        String::from($text);
    };
}
/*macro_rules! some_fstr {
    ($text:expr) => {
        Some(String::from($text));
    };
}*/

fn main() {
    
    //execute!(stdout(), Clear(ClearType::All)).unwrap();
    execute!(stdout(), SetTitle("TASD-Edit")).unwrap();
    
    let cli_state = parse_args();
    let tasd_file = std::fs::read(cli_state.tasd_path).unwrap();
    let start = Instant::now();
    let mut tasd = TasdMovie::new(&tasd_file).unwrap();
    let end = Instant::now();
    println!("Parse Time: {:.9} seconds\n", (end - start).as_secs_f64());
    
    while !main_menu(&mut tasd) {}
    
    exit(false, 0);
}

fn main_menu(tasd: &mut TasdMovie) -> bool {
    let selection = cli_selection(
        [
            fstr!("Exit/Quit"),
            fstr!("Modify a packet"),
            fstr!("Add a new packet"),
            fstr!("Remove a packet"),
            fstr!("Display all packets"),
        ].iter(), Some(fstr!("What would you like to do?\n")), Some(fstr!("Option[0]: "))
    );
    
    match selection {
        0 => true,
        1 => {
            edit_menu(tasd);
            false
        },
        2 => {
            add_menu(tasd);
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

fn edit_menu(tasd: &mut TasdMovie) {
    
}

fn add_menu(tasd: &mut TasdMovie) -> bool {
/*    let mut options = Vec::<String>::new();
    options.push(fstr!("Return to main menu"));
    
    let mut keypairs = Vec::<(u8, u8)>::new();
    for i in 0..=255 {
        for j in 0..=255 {
            let opt = key_description_lut(&[i, j]);
            if opt.is_some() {
                let opt = opt.unwrap();
                options.push(format!("{} - {}", opt.0, opt.1));
                keypairs.push((i, j));
            }
        }
    }
    let selection = cli_selection(options.iter(), Some(fstr!("Select the packet you'd like to add.\n")), Some(fstr!("Packet index[0]: ")));
    if selection != 0 {
        use spec::*;
        use spec::Packet::*;
        
        let pair = keypairs.get(selection - 1).unwrap();
        let patt: &[u8] = &[pair.0, pair.1];
        let packet = match patt {
            CONSOLE_TYPE => {
                let selection = cli_selection(Vec::from([
                    fstr!("Return to menu"), fstr!("NES"), fstr!("SNES"), fstr!("N64"), fstr!("GC"), fstr!("GB"), fstr!("GBC"), fstr!("GBA"), fstr!("Genesis"), fstr!("A2600"),
                ]).iter(), None, Some(fstr!("Console Type[0]: ")));
                if selection == 0 {
                    return false;
                }
                let payload = &[selection as u8];
                
                ConsoleType(PacketRaw::new(patt.to_vec(), PayloadSize::from_payload(payload), payload.to_vec()), payload[0])
            },
            CONSOLE_REGION => {
                let selection = cli_selection(Vec::from([
                    fstr!("Return to menu"), fstr!("NTSC"), fstr!("PAL")
                ]).iter(), None, some_fstr!("Console Region[0]: "));
                if selection == 0 {
                    return false;
                }
                let payload = &[selection as u8];
                
                ConsoleRegion(PacketRaw::new(patt.to_vec(), PayloadSize::from_payload(payload), payload.to_vec()), payload[0])
            },
            GAME_TITLE => {
                let text = cli_read(some_fstr!("Game title: "));
                if text.is_err() { return false; }
                let text = text.unwrap();
                
                GameTitle(PacketRaw::new(patt.to_vec(), PayloadSize::from_payload(text.as_bytes()), text.as_bytes().to_vec()), text)
            },
            AUTHOR => {
                let text = cli_read(some_fstr!("Author: "));
                if text.is_err() { return false; }
                let text = text.unwrap();
                
                Author(PacketRaw::new(patt.to_vec(), PayloadSize::from_payload(text.as_bytes()), text.as_bytes().to_vec()), text)
            },
            CATEGORY => {
                let text = cli_read(some_fstr!("Category: "));
                if text.is_err() { return false; }
                let text = text.unwrap();
                
                Category(PacketRaw::new(patt.to_vec(), PayloadSize::from_payload(text.as_bytes()), text.as_bytes().to_vec()), text)
            },
            EMULATOR_NAME => {
                let text = cli_read(some_fstr!("Emulator name: "));
                if text.is_err() { return false; }
                let text = text.unwrap();
                
                EmulatorName(PacketRaw::new(patt.to_vec(), PayloadSize::from_payload(text.as_bytes()), text.as_bytes().to_vec()), text)
            },
            EMULATOR_VERSION => {
                let text = cli_read(some_fstr!("Emulator version: "));
                if text.is_err() { return false; }
                let text = text.unwrap();
                
                EmulatorVersion(PacketRaw::new(patt.to_vec(), PayloadSize::from_payload(text.as_bytes()), text.as_bytes().to_vec()), text)
            },
            TAS_LAST_MODIFIED => {
                let text = cli_read(some_fstr!("TAS last modified (epoch seconds or YYYY-MM-DD): "));
                if text.is_err() { return false; }
                let text = text.unwrap();
                let mut epoch = 0i64;
                let parse_attempt = NaiveDate::parse_from_str(&text, "%Y-%m-%d");
                if parse_attempt.is_ok() {
                    let parsed = parse_attempt.unwrap();
                    let date = Date::<Utc>::from_utc(parsed, Utc);
                    epoch = date.and_time(NaiveTime::from_hms(0,0,0)).unwrap().timestamp();
                } else {
                    let parse_attempt = i64::from_str(&text);
                    if parse_attempt.is_err() { return false; }
                    epoch = parse_attempt.unwrap();
                }
                let payload = spec::expand_i64(epoch, 8);
                
                TasLastModified(PacketRaw::new(patt.to_vec(), PayloadSize::from_payload(&payload), payload), epoch)
            },
            TOTAL_FRAMES => {
                let text = cli_read(some_fstr!("Total frames: "));
                if text.is_err() { return false; }
                let parse_attempt = u32::from_str(&text.unwrap());
                if parse_attempt.is_err() { return false; }
                let frames = parse_attempt.unwrap();
                let payload = spec::expand_usize(frames as usize, 4);
                
                TotalFrames(PacketRaw::new(patt.to_vec(), PayloadSize::from_payload(&payload), payload), frames)
            },
            RERECORDS => {
                let text = cli_read(some_fstr!("Total frames: "));
                if text.is_err() { return false; }
                let parse_attempt = u32::from_str(&text.unwrap());
                if parse_attempt.is_err() { return false; }
                let rerecords = parse_attempt.unwrap();
                let payload = spec::expand_usize(rerecords as usize, 4);
                
                Rerecords(PacketRaw::new(patt.to_vec(), PayloadSize::from_payload(&payload), payload), rerecords)
            },
            SOURCE_LINK => {
                let text = cli_read(some_fstr!("Source link (url string): "));
                if text.is_err() { return false; }
                let text = text.unwrap();
                
                SourceLink(PacketRaw::new(patt.to_vec(), PayloadSize::from_payload(text.as_bytes()), text.as_bytes().to_vec()), text)
            },
            BLANK_FRAMES => {
                let text = cli_read(some_fstr!("Blank frames (number from -32767 to +32767): "));
                if text.is_err() { return false; }
                let parse_attempt = i16::from_str(&text.unwrap());
                if parse_attempt.is_err() { return false; }
                let frames = parse_attempt.unwrap();
                let payload = spec::expand_usize(frames as usize, 2);
                
                BlankFrames(PacketRaw::new(patt.to_vec(), PayloadSize::from_payload(&payload), payload), frames)
            },
            VERIFIED => {
                let text = cli_read(some_fstr!("Verified (number; 0 = false, 1 = true): "));
                if text.is_err() { return false; }
                let parse_attempt = u8::from_str(&text.unwrap());
                if parse_attempt.is_err() { return false; }
                let verified = parse_attempt.unwrap();
                let payload = [verified].to_vec();
                
                Verified(PacketRaw::new(patt.to_vec(), PayloadSize::from_payload(&payload), payload), verified)
            },
            //TODO: MEMORY_INIT =>      (),
            /*LATCH_FILTER =>     (),
            CLOCK_FILTER =>     (),
            OVERREAD =>         (),
            DPCM =>             (),
            GAME_GENIE_CODE =>  (),
            INPUT_CHUNKS =>     (),
            TRANSITION =>       (),
            LAG_FRAME_CHUNK =>  (),*/
            _ => Packet::Unsupported(PacketRaw::default())
        };
        match packet { Unsupported(_) => { return false; }, _ => () }
        
        tasd.packets.push(packet);
        
        dump_modified(tasd);
        
        println!("New packet added to file!\n");
        return false;
    }
    
    */
    true
}

fn remove_menu(tasd: &mut TasdMovie) -> bool {
    let mut options = Vec::<String>::new();
    options.push(fstr!("Return to main menu"));
    for packet in &tasd.packets {
        options.push(packet.get_packet_spec().to_string());
    }
    
    let selection = cli_selection(options.iter(), Some(fstr!("Select the packet you wish to remove.\n")), Some(fstr!("Packet index[0]: ")));
    if selection != 0 {
        println!("Packet removed.\n");
        tasd.packets.remove(selection - 1);
        return false;
    }
    
    true
}

fn display_packets(tasd: &TasdMovie) {
    println!("Version: {:#06X}, Key Width: {}", tasd.version, tasd.key_width);
    let padding = ((tasd.packets.len() as f32).log10() as usize) + 1;
    for (i, packet) in tasd.packets.iter().enumerate() {
        match packet {
            /*spec::Packet::InputChunks(_, _, _) => {
                //println!("[{:padding$.0}]: {}", i, packet, padding=padding);
            },*/
            
            _ => {
                println!("[{:padding$.0}]: {}: {}", i, packet.get_packet_spec().name, packet.formatted_payload(), padding=padding);
            }
        }
    }
    println!();
}

/*fn dump_modified(tasd: &mut TasdMovie) {
    for packet in tasd.packets.iter_mut() {
        match packet {
            Packet::DumpLastModified(raw, val) => {
                let epoch = Utc::now().timestamp();
                let payload = spec::expand_i64(epoch, 8);
                raw.payload = payload;
                *val = epoch;
            },
            _ => ()
        }
    }
}*/

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
        println!("Created new file: {}\n", path.to_string_lossy());
    } else {
        println!("Existing file found.\n");
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
