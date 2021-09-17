enum Stmt {
    Inc(usize),
    Dec(usize),
    IsZero(usize, usize),
    Jump(usize),
    Stop
}

fn main() {
    let filename = match std::env::args().skip(1).next() {
        Some(name) => name,
        None => eprintln!("error: no file provided.\nUsage: <filename>"),
    };

    let program = std::fs::read_to_string(filename).unwrap();
    let statements = parse(&program);
}

fn parse(text: &str) -> Result<Vec<Stmt>, String> {
    text.lines().map(parse_line).collect()
}


fn parse_line(line: &str) -> Result<Stmt, String> {
    const NO_REGISTER: fn() -> String = || "No register".to_string();
    const NO_LINE_NUMBER: fn() -> String = || "No line number".to_string();
    const EMPTY_LINE: fn() -> String = || "Empty line not allowed".to_string();


    let mut iter = line.split_ascii_whitespace();
    let first = iter.next().ok_or_else(EMPTY_LINE)?;

    Ok(match first {
        "INC" => {
            let register = iter.next().ok_or_else(NO_REGISTER)?.parse()?;
            Stmt::Inc(register)
        }
        "DEC" => {
            let register = iter.next().ok_or_else(NO_REGISTER)?.parse()?;
            Stmt::Dec(register)
        }
        "IS_ZERO" => {
            let register = iter.next().ok_or_else(NO_REGISTER)?.parse()?;
            let line_number = iter.next().ok_or_else(NO_LINE_NUMBER)?.parse()?;
            Stmt::IsZero(register, line_number)
        }
        "JUMP" => {
            let line_number = iter.next().ok_or_else(NO_LINE_NUMBER)?.parse()?;
            Stmt::Jump(line_number)
        }
        "STOP" => Stmt::Stop,
        stmt => return Err(format!("Illegal instruction: '{}'", stmt)),
    })
}
