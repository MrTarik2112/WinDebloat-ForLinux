use std::io::{stdout, IsTerminal};

fn main() {
    println!("stdout is terminal: {}", stdout().is_terminal());
    println!("stderr is terminal: {}", std::io::stderr().is_terminal());
    println!("stdin is terminal: {}", std::io::stdin().is_terminal());
}
