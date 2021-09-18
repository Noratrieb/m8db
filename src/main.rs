mod db;
mod stmt;

fn main() {
    println!(
        "m8db - M8 Debugger
(C) Nilstrieb (https://github.com/Nilstrieb/m8db)
Type 'help' for help
    "
    );

    db::start(std::env::args().nth(1));
}
