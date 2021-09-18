use std::collections::HashMap;
use std::fmt::Formatter;
use std::num::ParseIntError;

/// A span referencing the line where a statement came from. Starts at 0
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Span(pub usize);

impl Span {
    pub fn line_number(&self) -> usize {
        self.0 + 1
    }
}

/// A line number, starts at 1
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct LineNumber(pub usize);

impl LineNumber {
    pub fn span(&self) -> Span {
        Span(self.0 - 1)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Stmt {
    Inc(usize),
    Dec(usize),
    IsZero(usize, usize),
    Jump(usize),
    Stop,
}

#[derive(Debug, Clone)]
pub struct Code<'a> {
    pub stmts: Vec<Stmt>,
    /// Has the same length as `stmts`, points to line numbers where the instructions come from
    pub span: Vec<Span>,
    pub code_lines: Vec<&'a str>,
}

enum IrStmt<'a> {
    Inc(usize),
    Dec(usize),
    IsZeroLabel(usize, &'a str),
    IsZeroLine(usize, LineNumber),
    JumpLabel(&'a str),
    JumpLine(LineNumber),
    Label(&'a str),
    Stop,
}

pub fn parse(text: &str) -> Result<Code, String> {
    let mut labels = HashMap::new();

    let mut statements = Vec::new();
    let mut statement_number = 0;

    let code_lines = text.lines().collect::<Vec<_>>();

    for (line_index, line) in code_lines.iter().enumerate() {
        if line.split_whitespace().next().is_none() {
            continue;
        }
        let result = parse_line(line);
        match result {
            Ok(IrStmt::Label(name)) => {
                labels.insert(name, statement_number);
            }
            Ok(stmt) => {
                statement_number += 1;
                statements.push((stmt, Span(line_index)));
            }
            Err(msg) => return Err(format!("error on line '{}': {}", line_index - 1, msg)),
        }
    }

    let result: Result<Vec<(Stmt, Span)>, String> = statements
        .iter()
        .map(|(stmt, span)| match *stmt {
            IrStmt::Inc(r) => Ok((Stmt::Inc(r), *span)),
            IrStmt::Dec(r) => Ok((Stmt::Dec(r), *span)),
            IrStmt::IsZeroLine(r, line_number) => Ok((
                Stmt::IsZero(
                    r,
                    match statements
                        .iter()
                        .position(|(_, stmt_span)| stmt_span.line_number() == line_number.0)
                    {
                        Some(stmt_number) => stmt_number,
                        None => {
                            return Err(format!(
                                "Referencing line '{}' on line '{}': {}, out of bounds",
                                line_number.0,
                                span.line_number(),
                                code_lines[span.0]
                            ))
                        }
                    },
                ),
                *span,
            )),
            IrStmt::JumpLine(line_number) => Ok((
                Stmt::Jump(
                    match statements
                        .iter()
                        .position(|(_, stmt_span)| stmt_span.line_number() == line_number.0)
                    {
                        Some(stmt_number) => stmt_number,
                        None => {
                            return Err(format!(
                                "Referencing line '{}' on line '{}': {}, out of bounds",
                                line_number.0,
                                span.line_number(),
                                code_lines[span.0]
                            ))
                        }
                    },
                ),
                *span,
            )),
            IrStmt::IsZeroLabel(r, label) => Ok((
                Stmt::IsZero(
                    r,
                    match labels.get(label) {
                        Some(line) => *line,
                        None => {
                            return Err(format!(
                                "Label '{}' not found on line '{}'",
                                label,
                                span.line_number()
                            ))
                        }
                    },
                ),
                *span,
            )),
            IrStmt::JumpLabel(label) => Ok((
                Stmt::Jump(match labels.get(label) {
                    Some(line) => *line,
                    None => {
                        return Err(format!(
                            "Label '{}' not found on line {}",
                            label,
                            span.line_number()
                        ))
                    }
                }),
                *span,
            )),
            IrStmt::Stop => Ok((Stmt::Stop, *span)),
            IrStmt::Label(_) => unreachable!(),
        })
        .collect();

    result.map(|vec| {
        let (stmts, span) = vec.iter().cloned().unzip();
        Code {
            stmts,
            span,
            code_lines,
        }
    })
}

fn parse_line(line: &str) -> Result<IrStmt, String> {
    let no_register = || "No register provided".to_string();
    let no_label_or_line_number = || "No label or line number provided".to_string();
    let display_err = |parse_err: ParseIntError| parse_err.to_string();

    let mut iter = line.split_whitespace();
    let first = iter.next().expect("Empty lines filtered out");

    Ok(match first {
        "INC" => {
            let register = iter
                .next()
                .ok_or_else(no_register)?
                .parse()
                .map_err(display_err)?;
            IrStmt::Inc(register)
        }
        "DEC" => {
            let register = iter
                .next()
                .ok_or_else(no_register)?
                .parse()
                .map_err(display_err)?;
            IrStmt::Dec(register)
        }
        "IS_ZERO" => {
            let register = iter
                .next()
                .ok_or_else(no_register)?
                .parse()
                .map_err(display_err)?;
            let jump_target = iter.next().ok_or_else(no_label_or_line_number)?;
            if let Ok(line_number) = jump_target.parse::<usize>() {
                IrStmt::IsZeroLine(register, LineNumber(line_number))
            } else {
                IrStmt::IsZeroLabel(register, jump_target)
            }
        }
        "JUMP" => {
            let jump_target = iter.next().ok_or_else(no_label_or_line_number)?;
            if let Ok(line_number) = jump_target.parse::<usize>() {
                IrStmt::JumpLine(LineNumber(line_number))
            } else {
                IrStmt::JumpLabel(jump_target)
            }
        }
        "STOP" => IrStmt::Stop,
        stmt => {
            if stmt.starts_with('.') {
                IrStmt::Label(&stmt[1..])
            } else {
                return Err(format!("Illegal instruction: '{}'", stmt));
            }
        }
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
