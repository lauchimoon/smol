pub fn error(s: &str) -> String {
    let mut buf = String::from("\x1b[1;91m");
    buf.push_str(s);
    buf.push_str("\x1b[0m");
    buf
}

pub fn bold(s: &str) -> String {
    let mut buf = String::from("\x1b[1m");
    buf.push_str(s);
    buf.push_str("\x1b[0m");
    buf
}
