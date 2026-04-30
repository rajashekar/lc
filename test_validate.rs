use std::path::{Path, Component};

fn is_safe_filename(name: &str) -> bool {
    let path = Path::new(name);
    // Don't allow absolute paths, or paths containing ".." or current directory "."
    for component in path.components() {
        match component {
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => return false,
            // Component::CurDir is technically "." which is harmless but usually unnecessary
            // Let's just be strict and only allow Normal components
            Component::CurDir => return false,
            Component::Normal(_) => {}
        }
    }
    true
}

fn main() {
    println!("foo/bar: {}", is_safe_filename("foo/bar"));
    println!("/etc/passwd: {}", is_safe_filename("/etc/passwd"));
    println!("../../etc/passwd: {}", is_safe_filename("../../etc/passwd"));
    println!("providers/test.toml: {}", is_safe_filename("providers/test.toml"));
    println!("test.toml: {}", is_safe_filename("test.toml"));
}
