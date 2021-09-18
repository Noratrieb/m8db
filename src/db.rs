use crate::stmt::{Code, Stmt};
use std::io::Write;

#[derive(Debug, Clone)]
struct Vm<'a> {
    stmts: Vec<Stmt>,
    span: Vec<usize>,
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

    fn run(&mut self) -> VmState {
        loop {
            if let state @ (VmState::Break | VmState::Stop | VmState::OutOfBounds) = self.step() {
                return state;
            }
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum VmInstruction {
    Step,
    Run,
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
            VmInstruction::Run => match vm.run() {
                VmState::Stop => break,
                VmState::OutOfBounds => {
                    print_program(&vm);
                    print_registers(&vm);
                    eprintln!("error: Program ran out of bounds.");
                    return;
                }
                VmState::Run => unreachable!(),
                _ => {}
            },
            VmInstruction::Step => {
                if let VmState::Stop = vm.step() {
                    break;
                }
            }
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
                    Some(num) => match num.parse() {
                        Ok(num) => return VmInstruction::Break(num),
                        Err(_) => println!("Invalid argument provided"),
                    },
                    None => print_breakpoints(vm),
                },
                "set" => match parse_set_command(&mut iter) {
                    Some((reg, value)) => return VmInstruction::Set(reg, value),
                    None => println!("Invalid arguments provided"),
                },
                "c" | "continue" => return VmInstruction::Run,
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

    println!("Program: (pc = {}, line = {})", vm.pc, vm.span[vm.pc]);
    let lower_stmt = if vm.pc > 5 { vm.pc - 5 } else { 0 };
    let len = vm.stmts.len();
    let higher_stmt = if len < 5 { len } else { min(vm.pc + 5, len) };

    let lower_code = vm.span[lower_stmt];
    let higher_code = vm.span[higher_stmt - 1];

    for line_index in lower_code..higher_code {
        let code_line = vm.code_lines[line_index];
        if line_index == vm.span[vm.pc] {
            println!("> {}  {}", line_index + 1, code_line)
        } else {
            println!("{}  {}", line_index + 1, code_line);
        }
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
    continue (c) -- Run the program until the next breakpoint
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
