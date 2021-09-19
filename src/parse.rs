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

/// An index into a `Vm` `Stmt`
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct StmtIdx(pub usize);

/// A register index
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Register(pub usize);

#[derive(Debug, Copy, Clone)]
pub enum Stmt {
    Inc(Register),
    Dec(Register),
    IsZero(Register, StmtIdx),
    Jump(StmtIdx),
    Stop,
}

#[derive(Debug, Clone)]
pub struct Code<'a> {
    pub stmts: Vec<Stmt>,
    /// Has the same length as `stmts`, points to line numbers where the instructions come from
    pub span: Vec<Span>,
    pub code_lines: Vec<&'a str>,
    pub file_name: String,
}

#[derive(Debug, Clone)]
enum IrStmt<'a> {
    Inc(Register),
    Dec(Register),
    IsZeroLabel(Register, &'a str),
    IsZeroLine(Register, LineNumber),
    JumpLabel(&'a str),
    JumpLine(LineNumber),
    Label(&'a str),
    Stop,
    None,
}

#[derive(Debug)]
struct ParseErr {
    span: Span,
    inner: ParseErrInner,
}

impl ParseErr {
    fn new(span: Span, inner: ParseErrInner) -> Self {
        Self { span, inner }
    }
}

#[derive(Debug)]
pub enum ParseErrInner {
    OutOfBoundsLineRef(LineNumber),
    LabelNotFound(String),
    ParseIntErr(ParseIntError),
    NoRegister,
    NoLabelOrLine,
    IllegalStmt(String),
}

type StdResult<T, E> = std::result::Result<T, E>;
type Result<T> = StdResult<T, ParseErr>;

impl std::fmt::Display for ParseErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "error on line '{}': ", self.span.line_number())?;
        match &self.inner {
            ParseErrInner::OutOfBoundsLineRef(referenced) => {
                write!(f, "Referencing line '{}': out of bounds", referenced.0,)
            }
            ParseErrInner::LabelNotFound(label) => write!(f, "Label '{}' not found", label,),
            ParseErrInner::ParseIntErr(err) => write!(f, "{}", err),
            ParseErrInner::NoRegister => write!(f, "No register provided"),
            ParseErrInner::NoLabelOrLine => write!(f, "No label or line provided"),
            ParseErrInner::IllegalStmt(stmt) => write!(f, "Illegal statement: '{}'", stmt),
        }?;
        write!(f, ".")
    }
}

fn resolve_line_number(
    stmts: &[(IrStmt, Span)],
    number: LineNumber,
    span: Span,
) -> Result<StmtIdx> {
    match stmts
        .iter()
        .position(|(_, stmt_span)| stmt_span.line_number() == number.0)
    {
        Some(stmt_number) => Ok(StmtIdx(stmt_number)),
        None => Err(ParseErr::new(
            span,
            ParseErrInner::OutOfBoundsLineRef(number),
        )),
    }
}

fn resolve_label(labels: &HashMap<&str, StmtIdx>, span: Span, label: &str) -> Result<StmtIdx> {
    match labels.get(label) {
        Some(line) => Ok(*line),
        None => Err(ParseErr::new(
            span,
            ParseErrInner::LabelNotFound(label.to_owned()),
        )),
    }
}

pub fn parse(text: &str, file_name: String) -> StdResult<Code, String> {
    let mut labels = HashMap::new();

    let mut ir_statements = Vec::new();
    let mut statement_number = StmtIdx(0);

    let code_lines = text.lines().collect::<Vec<_>>();

    for (line_index, line) in code_lines.iter().enumerate() {
        let span = Span(line_index);
        let result = parse_line(span, line);
        match result {
            Ok(IrStmt::Label(name)) => {
                labels.insert(name, statement_number);
            }
            Ok(IrStmt::None) => {}
            Ok(stmt) => {
                statement_number.0 += 1;
                ir_statements.push((stmt, span));
            }
            Err(err) => return Err(err.to_string()),
        }
    }

    let statements: Result<Vec<_>> = ir_statements
        .iter()
        .filter(|stmt| !matches!(stmt, (IrStmt::None, _)))
        .map(|(stmt, span)| match *stmt {
            IrStmt::Inc(r) => Ok((Stmt::Inc(r), *span)),
            IrStmt::Dec(r) => Ok((Stmt::Dec(r), *span)),
            IrStmt::IsZeroLine(r, line_number) => Ok((
                Stmt::IsZero(r, resolve_line_number(&ir_statements, line_number, *span)?),
                *span,
            )),
            IrStmt::JumpLine(line_number) => Ok((
                Stmt::Jump(resolve_line_number(&ir_statements, line_number, *span)?),
                *span,
            )),
            IrStmt::IsZeroLabel(r, label) => Ok((
                Stmt::IsZero(r, resolve_label(&labels, *span, label)?),
                *span,
            )),
            IrStmt::JumpLabel(label) => {
                Ok((Stmt::Jump(resolve_label(&labels, *span, label)?), *span))
            }
            IrStmt::Stop => Ok((Stmt::Stop, *span)),
            IrStmt::Label(_) => unreachable!(),
            IrStmt::None => unreachable!(),
        })
        .collect();

    statements
        .map(|vec| {
            let (stmts, span) = vec.iter().cloned().unzip();
            Code {
                stmts,
                span,
                code_lines,
                file_name,
            }
        })
        .map_err(|err| err.to_string())
}

fn parse_line(span: Span, line: &str) -> Result<IrStmt> {
    let no_label_or_line_number = || ParseErr::new(span, ParseErrInner::NoLabelOrLine);

    let mut iter = line.split_whitespace();
    let first = iter.next();
    let first = match first {
        Some(first) => first,
        None => return Ok(IrStmt::None),
    };

    Ok(match first {
        "INC" => {
            let register = next_register(&mut iter, span)?;
            IrStmt::Inc(register)
        }
        "DEC" => {
            let register = next_register(&mut iter, span)?;
            IrStmt::Dec(register)
        }
        "IS_ZERO" => {
            let register = next_register(&mut iter, span)?;
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
            if let Some(stripped) = stmt.strip_prefix('.') {
                IrStmt::Label(stripped)
            } else if stmt.starts_with('#') {
                IrStmt::None
            } else {
                return Err(ParseErr::new(
                    span,
                    ParseErrInner::IllegalStmt(stmt.to_owned()),
                ));
            }
        }
    })
}

fn next_register<'a>(iter: &mut impl Iterator<Item = &'a str>, span: Span) -> Result<Register> {
    iter.next()
        .ok_or_else(|| ParseErr::new(span, ParseErrInner::NoRegister))?
        .parse()
        .map(|num| Register(num))
        .map_err(|parse_err: ParseIntError| {
            ParseErr::new(span, ParseErrInner::ParseIntErr(parse_err))
        })
}
