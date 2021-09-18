use crate::stmt::{Code, LineNumber, Span, Stmt};
use std::io::Write;

#[derive(Debug, Clone)]
struct Vm<'a> {
    stmts: Vec<Stmt>,
    span: Vec<Span>,
    code_lines: Vec<&'a str>,
    pc: usize,
    registers: Vec<usize>,
    breakpoints: Vec<usize>,
}

#[derive(Debug, Copy, Clone)]
enum VmState {
    Run,
    Break,
    Stop,
    OutOfBounds,
}

impl Vm<'_> {
    fn step(&mut self) -> VmState {
        let pc = self.pc;
        match self.stmts.get(pc).cloned() {
            Some(Stmt::Inc(r)) => self.registers[r] += 1,
            Some(Stmt::Dec(r)) => self.registers[r] -= 1,
            Some(Stmt::IsZero(r, line)) => {
                if self.registers[r] == 0 {
                    self.pc = line - 1;
                }
            }
            Some(Stmt::Jump(line)) => self.pc = line - 1,
            Some(Stmt::Stop) => return VmState::Stop,
            None => return VmState::OutOfBounds,
        }
        self.pc += 1;
        if self.breakpoints.contains(&self.pc) {
            VmState::Break
        } else {
            VmState::Run
        }
    }

    fn run(&mut self, time_kind: VmRunKind) -> VmState {
        let now = std::time::Instant::now();
        loop {
            if let state @ (VmState::Break | VmState::Stop | VmState::OutOfBounds) = self.step() {
                if let VmRunKind::WithTime = time_kind {
                    println!("Vm ran for {}ms.", now.elapsed().as_millis());
                }
                return state;
            }
        }
    }

    fn statement_at_span(&self, search_span: Span) -> Option<usize> {
        self.span.iter().position(|span| *span >= search_span)
    }
}

#[derive(Debug, Copy, Clone)]
enum VmRunKind {
    WithTime,
    WithoutTime,
}

#[derive(Debug, Copy, Clone)]
enum VmInstruction {
    Step,
    Run(VmRunKind),
    Break(usize),
    Set(usize, usize),
}

pub fn run(code: Code) {
    let max_register_index = max_register(&code.stmts);
    let mut vm = Vm {
        stmts: code.stmts,
        span: code.span,
        code_lines: code.code_lines,
        pc: 0,
        registers: vec![0; max_register_index + 1],
        breakpoints: vec![],
    };

    loop {
        match debug_input(&vm) {
            VmInstruction::Run(time_kind) => match vm.run(time_kind) {
                VmState::Stop => break,
                VmState::OutOfBounds => {
                    print_program(&vm);
                    print_registers(&vm);
                    eprintln!("error: Program ran out of bounds.");
                    return;
                }
                VmState::Run => unreachable!("Program still running after returning from run"),
                _ => {}
            },
            VmInstruction::Step => match vm.step() {
                VmState::Stop => break,
                VmState::OutOfBounds => {
                    print_program(&vm);
                    print_registers(&vm);
                    eprintln!("error: Program ran out of bounds.");
                    return;
                }
                _ => {}
            },
            VmInstruction::Break(line) => {
                let position = vm.breakpoints.iter().position(|point| *point == line);
                match position {
                    None => vm.breakpoints.push(line),
                    Some(pos) => {
                        vm.breakpoints.remove(pos);
                    }
                }
            }
            VmInstruction::Set(r, value) => vm.registers[r] = value,
        }
    }
}

fn debug_input(vm: &Vm) -> VmInstruction {
    loop {
        let input = get_input();
        let mut iter = input.split_ascii_whitespace();
        if let Some(str) = iter.next() {
            match str {
                "r" | "register" => print_registers(vm),
                "p" | "program" => print_program(vm),
                "h" | "?" | "help" => print_help(),
                "b" | "break" => match iter.next() {
                    Some(line_number) => match line_number.parse::<usize>() {
                        Ok(line_number) => {
                            let stmt_pos =
                                match vm.statement_at_span(LineNumber(line_number).span()) {
                                    Some(pos) => pos,
                                    None => {
                                        println!(
                                            "Line number '{}' out of bounds for length {}.",
                                            line_number,
                                            vm.code_lines.len()
                                        );
                                        continue;
                                    }
                                };
                            return VmInstruction::Break(stmt_pos);
                        }
                        Err(_) => println!("Invalid argument provided"),
                    },
                    None => print_breakpoints(vm),
                },
                "set" => match parse_set_command(&mut iter) {
                    Some((reg, value)) => return VmInstruction::Set(reg, value),
                    None => println!("Invalid arguments provided"),
                },
                "c" | "continue" => {
                    if let Some("time") = iter.next() {
                        return VmInstruction::Run(VmRunKind::WithTime);
                    }
                    return VmInstruction::Run(VmRunKind::WithoutTime);
                }
                "s" | "step" => return VmInstruction::Step,
                _ => {}
            }
        }
    }
}

fn parse_set_command<'a>(iter: &mut impl Iterator<Item = &'a str>) -> Option<(usize, usize)> {
    let reg: usize = iter.next().and_then(|reg| reg.parse().ok())?;
    let value: usize = iter.next().and_then(|value| value.parse().ok())?;
    Some((reg, value))
}

fn max_register(stmts: &[Stmt]) -> usize {
    stmts
        .iter()
        .map(|stmt| match stmt {
            Stmt::Inc(r) => *r,
            Stmt::Dec(r) => *r,
            Stmt::IsZero(r, _) => *r,
            Stmt::Jump(_) => 0,
            Stmt::Stop => 0,
        })
        .max()
        .unwrap_or(0)
}

fn print_registers(vm: &Vm) {
    println!("Registers:");
    for (i, r) in vm.registers.iter().enumerate() {
        println!("{: >4} : {}", i, r);
    }
}

fn print_program(vm: &Vm) {
    use std::cmp::min;

    if let Some(span_pc) = vm.span.get(vm.pc) {
        println!(
            "Program: (pc = {}, line = {})",
            vm.pc,
            span_pc.line_number()
        );

        let lower = span_pc.0.saturating_sub(5);
        let higher = min(vm.code_lines.len(), span_pc.0 + 6);

        for line_index in lower..higher {
            let code_line = vm.code_lines[line_index];
            if line_index == span_pc.0 {
                println!("> {}  {}", Span(line_index).line_number(), code_line);
            } else {
                println!("{}  {}", Span(line_index).line_number(), code_line);
            }
        }
    } else {
        println!("Reached the end of the program.");
    }
}

fn print_breakpoints(vm: &Vm) {
    println!(
        "Breakpoints:
    {}
    ",
        vm.breakpoints
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<String>>()
            .join(", ")
    );
}

fn print_help() {
    println!(
        "List of commands and their aliases:

    step (s) -- Steps the program forward by one step
    set <register> <value> -- Sets a register to a value
    break <line> (b) -- Set a breakpoint to a line, use again to toggle
    continue (c) (time) -- Run the program until the next breakpoint, add 'time' to display execution time
    register (r) -- Shows the contents of the registers
    program (p) -- Shows where the program currently is
    help (h, ?) -- Shows this help page
    "
    );
}

fn get_input() -> String {
    let mut input_buf = String::new();
    print!("(m8db) ");
    std::io::stdout().flush().unwrap();
    std::io::stdin().read_line(&mut input_buf).unwrap();
    input_buf.trim().to_owned()
}
