pub fn string_to_hex(s: &String) -> String {
    s.bytes().map(char_to_hex).fold(String::new(), concat)
}

fn char_to_hex(c: u8) -> String {
    format!("\\{:X}", c)
}

fn concat(l: String, r: String) -> String {
    format!("{}{}", l, r)
}
