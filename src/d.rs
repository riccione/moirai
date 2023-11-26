use std::str;

// for debug only
// convert bytes to utf8 and prints
pub fn _print_bytes(buf: &[u8]) {
    let received = str::from_utf8(&buf);
    match received {
        Ok(x) => {
            println!("{x}");
        }
        Err(e) => {
            eprintln!("{e}");
        }
    }
}

// for debug only
pub fn _print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}
