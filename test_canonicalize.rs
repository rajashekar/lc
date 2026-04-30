use std::path::PathBuf;
use std::fs;

fn main() {
    let mut config_dir = PathBuf::from("./test_config");
    fs::create_dir_all(&config_dir).unwrap();
    config_dir = config_dir.canonicalize().unwrap();

    let bad_name1 = "../test_canonicalize.rs";
    let p1 = config_dir.join(bad_name1);

    let bad_name2 = "/etc/passwd";
    let p2 = config_dir.join(bad_name2);

    let good_name = "providers/anthropic.toml";
    let p3 = config_dir.join(good_name);

    println!("p1: {:?}, canonicalized starts_with? {:?}", p1, p1.canonicalize().map(|p| p.starts_with(&config_dir)));
    println!("p2: {:?}, canonicalized starts_with? {:?}", p2, p2.canonicalize().map(|p| p.starts_with(&config_dir)));
    println!("p3: {:?}, canonicalized starts_with? {:?}", p3, p3.canonicalize().map(|p| p.starts_with(&config_dir)));
}
