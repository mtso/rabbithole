use std::num::Wrapping;

pub fn hash(str: &String) -> String {
    let mut raw = Wrapping(274777u64);
    let mut bytes: [u8; 6] = [0; 6];

    for b in str.as_bytes().iter() {
        raw = raw * Wrapping(*b as u64) + Wrapping(33u64);
    }

    for i in 0..6 {
        let rem = raw.0 % 16;
        raw = raw / Wrapping(16u64);
        bytes[i] = rem as u8; //table[rem as usize];
    }

    format!(
        "#{:x}{:x}{:x}{:x}{:x}{:x}",
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5]
    )
}
