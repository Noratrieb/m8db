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
    text.lines()
        .filter(|line| line.split_whitespace().next().is_some())
        .map(parse_line)
        .collect()
}

fn parse_line(line: &str) -> Result<Stmt, String> {
    let no_register = || "No register".to_string();
    let no_line_number = || "No line number".to_string();
    let empty_line = || "Empty line not allowed".to_string();
    let display_err = |parse_err: ParseIntError| parse_err.to_string();

    let mut iter = line.split_ascii_whitespace();
    let first = iter.next().ok_or_else(empty_line)?;

    Ok(match first {
        "INC" => {
            let register = iter
                .next()
                .ok_or_else(no_register)?
                .parse()
                .map_err(display_err)?;
            Stmt::Inc(register)
        }
        "DEC" => {
            let register = iter
                .next()
                .ok_or_else(no_register)?
                .parse()
                .map_err(display_err)?;
            Stmt::Dec(register)
        }
        "IS_ZERO" => {
            let register = iter
                .next()
                .ok_or_else(no_register)?
                .parse()
                .map_err(display_err)?;
            let line_number = iter
                .next()
                .ok_or_else(no_line_number)?
                .parse()
                .map_err(display_err)?;
            Stmt::IsZero(register, line_number)
        }
        "JUMP" => {
            let line_number = iter
                .next()
                .ok_or_else(no_line_number)?
                .parse()
                .map_err(display_err)?;
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
