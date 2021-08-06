use std::path::Path;
use std::time::Instant;

mod spec;


fn main() {
    let test_file = std::fs::read(Path::new("test.tasd")).unwrap();
    let start = Instant::now();
    let parsed = spec::TasdMovie::new(&test_file).unwrap();
    let end = Instant::now();
    println!("Parse Time: {:.9} seconds", (end - start).as_secs_f64());
    
    println!("Version: {:#06X}, Key Width: {}", parsed.version, parsed.key_width);
    for packet in parsed.packets {
        println!("{}", packet);
    }
}
