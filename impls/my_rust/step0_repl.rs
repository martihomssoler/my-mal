use std::io::Write;

fn read(input: &str) -> &str {
    input
}

fn eval(input: &str) -> &str {
    input
}

fn print(input: &str) -> &str {
    input
}

fn rep(input: &str) -> &str {
    let ast = read(input);
    let result = eval(ast);
    print(result)
}

pub fn main() {
    loop {
        // print prompt
        print!("user> ");
        // get input line
        let Ok(input) = get_line() else {
            break;
        };
        // call rep
        let output = rep(&input);
        // print result
        println!("{output}");
    }
}

fn get_line() -> std::io::Result<String> {
    std::io::stdout().flush()?;
    let mut buffer = String::new();
    let stdin = std::io::stdin(); // We get `Stdin` here.
    stdin.read_line(&mut buffer)?;
    Ok(buffer.trim_end().to_owned())
}
