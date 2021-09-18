use crate::stmt::Stmt;
use std::io::Write;

#[derive(Debug, Clone)]
struct Vm {
    stmts: Vec<Stmt>,
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

impl Vm {
    fn step(&mut self) -> VmState {
        let pc = self.pc;
        self.pc += 1;
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

pub fn run(stmts: Vec<Stmt>) {
    let max_register_index = max_register(&stmts);
    let mut vm = Vm {
        stmts,
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

    println!("Program:");
    let lower = if vm.pc > 5 { vm.pc - 5 } else { 0 };
    let len = vm.stmts.len();
    let higher = if len < 5 { len } else { min(vm.pc + 5, len) };

    for i in lower..higher {
        let stmt = vm.stmts[i];
        if i == vm.pc {
            println!("> {}  {}", i, stmt)
        } else {
            println!("{}  {}", i, stmt);
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
