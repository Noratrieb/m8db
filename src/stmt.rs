use std::fmt::Formatter;
use std::num::ParseIntError;

#[derive(Debug, Copy, Clone)]
pub enum Stmt {
    Inc(usize),
    Dec(usize),
    IsZero(usize, usize),
    Jump(usize),
    Stop,
}

pub fn parse(text: &str) -> Result<Vec<Stmt>, String> {
    text.lines().map(parse_line).collect()
}

fn parse_line(line: &str) -> Result<Stmt, String> {
    const NO_REGISTER: fn() -> String = || "No register".to_string();
    const NO_LINE_NUMBER: fn() -> String = || "No line number".to_string();
    const EMPTY_LINE: fn() -> String = || "Empty line not allowed".to_string();
    const DISPLAY_ERR: fn(ParseIntError) -> String = |parse_err| parse_err.to_string();

    let mut iter = line.split_ascii_whitespace();
    let first = iter.next().ok_or_else(EMPTY_LINE)?;

    Ok(match first {
        "INC" => {
            let register = iter
                .next()
                .ok_or_else(NO_REGISTER)?
                .parse()
                .map_err(DISPLAY_ERR)?;
            Stmt::Inc(register)
        }
        "DEC" => {
            let register = iter
                .next()
                .ok_or_else(NO_REGISTER)?
                .parse()
                .map_err(DISPLAY_ERR)?;
            Stmt::Dec(register)
        }
        "IS_ZERO" => {
            let register = iter
                .next()
                .ok_or_else(NO_REGISTER)?
                .parse()
                .map_err(DISPLAY_ERR)?;
            let line_number = iter
                .next()
                .ok_or_else(NO_LINE_NUMBER)?
                .parse()
                .map_err(DISPLAY_ERR)?;
            Stmt::IsZero(register, line_number)
        }
        "JUMP" => {
            let line_number = iter
                .next()
                .ok_or_else(NO_LINE_NUMBER)?
                .parse()
                .map_err(DISPLAY_ERR)?;
            Stmt::Jump(line_number)
        }
        "STOP" => Stmt::Stop,
        stmt => return Err(format!("Illegal instruction: '{}'", stmt)),
    })
}

impl std::fmt::Display for Stmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Stmt::Inc(r) => write!(f, "INC {}", r)?,
            Stmt::Dec(r) => write!(f, "DEC {}", r)?,
            Stmt::IsZero(r, line) => write!(f, "IS_ZERO {} {}", r, line)?,
            Stmt::Jump(r) => write!(f, "JUMP {}", r)?,
            Stmt::Stop => write!(f, "STOP")?,
        }
        Ok(())
    }
}
