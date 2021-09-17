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
}

impl Vm {
    fn step(&mut self) -> VmState {
        let pc = self.pc;
        self.pc += 1;
        match self.stmts[pc] {
            Stmt::Inc(r) => self.registers[r] += 1,
            Stmt::Dec(r) => self.registers[r] -= 1,
            Stmt::IsZero(r, line) => {
                if self.registers[r] == 0 {
                    self.pc = line;
                }
            }
            Stmt::Jump(line) => self.pc = line,
            Stmt::Stop => return VmState::Stop,
        }
        if self.breakpoints.contains(&self.pc) {
            VmState::Break
        } else {
            VmState::Run
        }
    }

    fn run(&mut self) -> VmState {
        while let VmState::Run = self.step() {}
        VmState::Break
    }
}

#[derive(Debug, Copy, Clone)]
enum VmInstruction {
    Step,
    Run,
    Break(usize),
}

pub fn run(stmts: Vec<Stmt>) {
    let max_register = max_register(&stmts);
    let mut vm = Vm {
        stmts,
        pc: 0,
        registers: vec![0; max_register],
        breakpoints: vec![],
    };

    loop {
        match debug_input(&mut vm) {
            VmInstruction::Run => match vm.run() {
                VmState::Stop => break,
                VmState::Run => unreachable!(),
                _ => {}
            },
            VmInstruction::Step => match vm.step() {
                VmState::Stop => break,
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
        }
    }
}

fn debug_input(vm: &Vm) -> VmInstruction {
    loop {
        let mut input_buf = String::new();
        print!("(m8db) ");
        std::io::stdout().flush().unwrap();
        std::io::stdin().read_line(&mut input_buf).unwrap();
        let input = input_buf.trim();
        let mut iter = input.split_ascii_whitespace();
        match iter.next() {
            Some(str) => match str {
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
                "c" | "continue" => return VmInstruction::Run,
                "s" | "step" => return VmInstruction::Step,
                _ => {}
            },
            None => {}
        }
    }
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
    use std::cmp::{max, min};

    println!("Program:");
    let lower = max(vm.pc, 5) - 5;
    let higher = min(vm.pc, vm.stmts.len() - 6) + 5;

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
    break <line> (b) -- Set a breakpoint to a line, use again to toggle
    continue (c) -- Run the program until the next breakpoint
    register (r) -- Shows the contents of the registers
    program (p) -- Shows where the program currently is
    help (h, ?) -- Shows this help page
    "
    );
}
