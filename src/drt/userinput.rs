use std::io::stdin;

pub fn ask(question: &str) -> char {
    loop {
        println!("{}", question);
        let mut line = String::new();
        stdin().read_line(&mut line).expect("No User Input");
        if ! line.is_empty() {
            match line.trim().chars().nth(0) {
                Some(ch) => return ch,
                _ => {}
            }
        }
    }
}
