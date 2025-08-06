use std::env;
use lc::input::MultiLineInput;

fn main() {
    // Enable debug logging
    env::set_var("LC_DEBUG_INPUT", "1");
    
    println!("Testing multi-line input:");
    println!("- Press Shift+Enter to add new lines");
    println!("- Press Enter to submit");
    println!("- Press Ctrl+C to cancel");
    println!();

    let mut input_handler = MultiLineInput::new();
    
    loop {
        match input_handler.read_input("Test") {
            Ok(input) => {
                if input.is_empty() {
                    println!("Input canceled or empty");
                    break;
                } else {
                    println!("You entered:");
                    println!("'{}'", input);
                    println!("Lines: {:?}", input.lines().collect::<Vec<_>>());
                    println!();
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                break;
            }
        }
    }
}
