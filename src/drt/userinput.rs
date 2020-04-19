use std::io::stdin;

pub fn ask(question: String) -> String {
    println!("{}", question);
    let mut line = String::new();
    stdin().read_line(&mut line).expect("No User Input");
    return line.trim().to_string();

    //BufReader::new(std::io::stdin()).read_line().unwrap_or("");
}
