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

    let program = std::fs::read_to_string(filename).unwrap();
    let statements = match stmt::parse(&program) {
        Ok(stmts) => stmts,
        Err(str) => {
            eprintln!("{}", str);
            return;
        }
    };
    db::run(statements);
}
