

pub fn to_u16(data: &[u8]) -> u16 {
    to_usize(data) as u16
}

pub fn to_i16(data: &[u8]) -> i16 {
    to_usize(data) as i16
}

pub fn to_u32(data: &[u8]) -> u32 {
    to_usize(data) as u32
}

pub fn to_u64(data: &[u8]) -> u64 {
    to_usize(data) as u64
}

pub fn to_i64(data: &[u8]) -> i64 {
    to_usize(data) as i64
}

pub fn to_usize(data: &[u8]) -> usize {
    let mut out = 0;
    
    for by in data {
        out = out << 8;
        out |= *by as usize;
    }
    
    out
}

pub fn to_bytes(mut number: usize, length: u8) -> Vec<u8> {
    let mut out = Vec::new();
    
    for _ in 0..length {
        out.insert(0, (number & 0xFF) as u8);
        number >>= 8;
    }
    
    out
}

pub fn print_slice(slice: &[u8]) {
    for byte in slice {
        print!("{:02X} ", byte);
    }
    println!("")
}

pub fn format_slice_hex(slice: &[u8]) -> String {
    let mut s = String::new();
    for byte in slice {
        s.push_str(&format!("{:02X} ", byte));
    }
    s.trim().to_owned()
}

pub fn format_slice_bin(slice: &[u8]) -> String {
    let mut s = String::new();
    for byte in slice {
        s.push_str(&format!("{:08b} ", byte));
    }
    s
}