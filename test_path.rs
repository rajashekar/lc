use std::path::PathBuf;

fn main() {
    let base = PathBuf::from("/home/user/.config/lc");
    println!("{:?}", base.join("test.txt"));
    println!("{:?}", base.join("../../etc/passwd"));
    println!("{:?}", base.join("/etc/passwd"));
}
