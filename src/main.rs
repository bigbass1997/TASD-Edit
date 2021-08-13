use std::path::{PathBuf, Path};
use std::time::Instant;
use regex::Regex;
use std::io::{stdout, Error, Write};
use crossterm::execute;
use crate::spec::{NEW_TASD_FILE, TasdMovie};
use crossterm::terminal::{SetTitle, Clear, ClearType};
use std::ffi::OsStr;
use strum::IntoEnumIterator;

mod spec;

fn main() {
    //execute!(stdout(), Clear(ClearType::All)).unwrap();
    execute!(stdout(), SetTitle("TASD-Edit")).unwrap();
    
    let cli_state = parse_args();
    let tasd_file = std::fs::read(cli_state.tasd_path).unwrap();
    let start = Instant::now();
    let mut tasd = spec::TasdMovie::new(&tasd_file).unwrap();
    let end = Instant::now();
    println!("Parse Time: {:.9} seconds\n", (end - start).as_secs_f64());
    
    while !main_menu(&mut tasd) {}
    
    exit(false, 0);
}

fn main_menu(tasd: &mut TasdMovie) -> bool {
    let selection = cli_selection(
        [
            String::from("Exit/Quit"),
            String::from("Modify a packet"),
            String::from("Add a new packet"),
            String::from("Remove a packet"),
            String::from("Display all packets"),
        ].iter(), Some(String::from("What would you like to do?\n")), Some(String::from("Option[0]: "))
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
    let mut options = Vec::<String>::new();
    options.push(String::from("Return to main menu"));
    for packet_kind in spec::Packet::iter() {
        println!("{}", packet_kind);
    }
    
    true
}

fn remove_menu(tasd: &mut TasdMovie) -> bool {
    let mut options = Vec::<String>::new();
    options.push(String::from("Return to main menu"));
    for packet in &tasd.packets {
        options.push(packet.to_string());
    }
    
    let selection = cli_selection(options.iter(), Some(String::from("Select the packet you wish to remove.\n")), Some(String::from("Packet index[0]: ")));
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
            spec::Packet::InputChunks(_, _, _) => {
                //println!("[{:padding$.0}]: {}", i, packet, padding=padding);
            },
            
            _ => {
                println!("[{:padding$.0}]: {}", i, packet, padding=padding);
            }
        }
    }
    println!();
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