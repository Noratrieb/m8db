mod db;
mod stmt;

fn main() {
    let filename = match std::env::args().skip(1).next() {
        Some(name) => name,
        None => {
            eprintln!("error: no file provided.\nUsage: <filename>");
            return;
        }
    };

    println!(
        "m8db - M8 Debugger
(C) Nilstrieb (https://github.com/Nilstrieb/m8db)
Type 'help' for help
    "
    );

    let program = std::fs::read_to_string(filename).unwrap();
    let statements = match stmt::parse(&program) {
        Ok(stmts) => stmts,
        Err(str) => {
            eprintln!("{}", str);
            return;
        }
    };
    db::run(statements);
    println!("Execution finished.");
}
