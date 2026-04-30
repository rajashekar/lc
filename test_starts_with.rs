use std::path::PathBuf;

fn main() {
    let config_dir = PathBuf::from("/home/user/.config/lc");

    let bad_name1 = "../../etc/passwd";
    let p1 = config_dir.join(bad_name1);

    let bad_name2 = "/etc/passwd";
    let p2 = config_dir.join(bad_name2);

    let good_name = "providers/anthropic.toml";
    let p3 = config_dir.join(good_name);

    println!("p1: {:?}, starts_with? {}", p1, p1.starts_with(&config_dir));
    println!("p2: {:?}, starts_with? {}", p2, p2.starts_with(&config_dir));
    println!("p3: {:?}, starts_with? {}", p3, p3.starts_with(&config_dir));
}
