use std::env;

pub fn bin_name() -> String {
    let name = env::args().next().unwrap();
    name.split('/').last().unwrap().to_string()
}
