use std::env;

pub fn bin_name() -> String {
    let name = env::args().next().unwrap();
    name.split('/').next_back().unwrap().to_string()
}
