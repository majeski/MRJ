pub fn string_to_hex(s: &String) -> String {
    s.bytes().map(char_to_hex).fold(String::new(), concat)
}

fn char_to_hex(c: u8) -> String {
    format!("\\{:X}", c)
}

fn concat(l: String, r: String) -> String {
    format!("{}{}", l, r)
}

pub fn join<T: Copy, F>(v: &Vec<T>, c: char, f: F) -> String
    where F: Fn(T) -> String
{
    let mut res = String::new();
    for e in v {
        res = if res.is_empty() {
            format!("{}", f(*e))
        } else {
            format!("{}{} {}", res, c, f(*e))
        };
    }
    res
}
