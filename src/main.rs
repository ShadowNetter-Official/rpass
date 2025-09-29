use tabled::{Tabled, Table, settings::{Style, object::Rows, Color}};
use std::{fs, env, process};
use serde::{Serialize, Deserialize};
use prompted::input;

const DEFAULT_FILE: &str = "passwords.json";


#[derive(Tabled, Debug, Serialize, Deserialize, Clone)]
struct Password {
    service: String,
    email: String,
    username: String,
    password: String,
    index: usize,
}

fn load(file: &String) -> Vec<Password> {
    let passwords = match fs::read_to_string(file) {
        Ok(passwords) => match serde_json::from_str(&passwords) {
            Ok(p) => p,
            Err(_) => Vec::<Password>::new(),
        },
        Err(_) => Vec::<Password>::new(),
    };
    passwords
}

fn index(mut passwords: Vec<Password>) -> Vec<Password> {
    let mut count: usize = 0;
    for i in &mut passwords {
        i.index = count;
        count+=1;
    }
    passwords
}


fn search(passwords: Vec<Password>) {
    let search_term = input!("search service: ");
    let mut results: Vec<Password> = Vec::new();
    for password in passwords {
        if password.service.to_lowercase() == search_term.to_lowercase() {
            results.push(password);
        }
    }
    if results.len() < 1 {
        println!("No results for {search_term}");
        return;
    }
    display_passwords(&results);
}

fn save(data: &Vec<Password>, file: &String) {
    let json = serde_json::to_string_pretty(data).unwrap();
    match fs::write(file, json) {
        Ok(_) => (),
        Err(e) => {
            println!("error writing to file: {file}\n {e}");
            return;
        },
    }
}

fn new_password(mut passwords: Vec<Password>) -> Vec<Password> {
    let s = input!("service: ");
    let p = input!("password: ");
    let u = input!("username: ");
    let e = input!("email: ");
    let i = passwords.len() + 1;
    let password = Password {service: s, email: e, username: u, password: p, index: i};
    passwords.push(password);
    passwords
}

fn remove_password(mut passwords: Vec<Password>) -> Vec<Password> {
    fn get_usize() -> usize {
        let input = input!("index: ");
        let num: usize = match input.parse::<usize>() {
            Ok(num) => num,
            Err(_) => {
                println!("invalid index");
                get_usize()
            },
        };
        num
    }
    let index = get_usize();
    passwords.remove(index);
    passwords
}

fn display_passwords(passwords: &Vec<Password>) {
    let mut table = Table::new(passwords);
    table.with(Style::modern());
    table.modify(Rows::first(), Color::BG_BLACK);
    println!("{table}");
}

fn rpass() {
    let file = get_file();
    help();
    let mut passwords = load(&file);
    let mut changes: usize = 0;
    loop {
        passwords = index(passwords);
        let command = input!(">> ");
        match &command as &str {
            "new" => {
                passwords = new_password(passwords);
                save(&passwords, &file);
            },
            "list" => display_passwords(&passwords),
            "quit" => quit(changes),
            "remove" => {
                passwords = remove_password(passwords);
                changes+=1;
            },
            "save" => {
                save(&passwords, &file);
                changes = 0;
            },
            "clear" => clear(),
            "search" => search(passwords.clone()),
            "forcequit" => {
                clear();
                break;
            },
            _ => help(),
        }
    }
}

fn get_file() -> String {
    let argv: Vec<String> = env::args().collect();
    if argv.len() < 2 {
        println!("No file specified, opening default: {DEFAULT_FILE}");
        return DEFAULT_FILE.to_string();
    }
    match &argv[1] as &str {
        "-h" | "--help" => {
            rhelp();
            process::exit(0);
        },
        _ => (),
    }
    (&argv[1]).to_string()
}

fn rhelp() {
    println!("rpass | a CLI password manager written in Rust 🦀\n");
    println!("usage: \n");
    println!("rpass <file.json> | if no file is found/provided passwords.json is created");
    println!("rpass -h, --help  | display this message");
}

fn help() {
    println!("");
    println!("new       | create new password");
    println!("list      | list passwords");
    println!("quit      | quit program");
    println!("forcequit | quit without saving");
    println!("save      | save changes");
    println!("clear     | clear screen");
    println!("remove    | remove password");
    println!("search    | search for service");
    println!("help      | display this message\n");
}


fn clear() {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
}

fn quit(changes: usize) {
    if changes < 1 {
        clear();
        process::exit(0);
    }
    println!("You have {changes} unsaved changes");
    println!("try: forcequit to quit without saving");
}

fn main() {
    rpass();
}
