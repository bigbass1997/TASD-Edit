use std::path::Path;

mod spec;


fn main() {
    let test_file = std::fs::read(Path::new("test.tasd")).unwrap();
    let parsed = spec::TasdMovie::new(&test_file);
    
    println!("{:?}", parsed);
}
