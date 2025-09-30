use tabled::{Tabled, Table, settings::{Style, object::{Rows, Columns}, Remove, Color}};
use std::{fs, env, process};
use serde::{Serialize, Deserialize};
use prompted::input;
use arboard::Clipboard;

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
    display_passwords(&results, false);
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

struct Clippy {
    clipboard: Clipboard
}

impl Clippy {
    fn init() -> Self {
        Self {
            clipboard: Clipboard::new().expect("error creating clipboard"),
        }
    }
    fn copy_password(&mut self, passwords: &Vec<Password>) {
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
        let pass = &passwords[index].password;
        match self.clipboard.set_text((&pass).to_string()) {
            Ok(_) => println!("copied {pass} to clipboard"),
            Err(e) => {
                println!("error copying password: {e}");
                println!("password was not copied");
            },
        }
    }
}

fn display_passwords(passwords: &Vec<Password>, hide: bool) {
    let mut table = Table::new(passwords);
    table.with(Style::modern());
    table.modify(Rows::first(), Color::BG_BLACK);
    if hide {
        table.with(Remove::column(Columns::one(3)));
    }
    println!("{table}");
}

fn rpass() {
    let (file, hide) = get_args();
    let mut clippy = Clippy::init();
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
            "list" => display_passwords(&passwords, hide),
            "quit" => quit(changes),
            "remove" => {
                passwords = remove_password(passwords);
                changes+=1;
            },
            "save" => {
                save(&passwords, &file);
                changes = 0;
            },
            "copy" => clippy.copy_password(&passwords),
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

fn get_args() -> (String, bool) {
    let argv: Vec<String> = env::args().collect(); 
    let mut hide: bool = false;
    let mut file: String = String::new();
    match argv.len() {
        1 => {
            // no args
            println!("No file specified, opening default: {DEFAULT_FILE}");
            return (DEFAULT_FILE.to_string(), hide);
        },
        2 => {
            // 1 arg
            match &argv[1] as &str {
                "-h" | "--help" => {
                    rhelp();
                    process::exit(0);
                },
                "--hidden" => {
                    println!("No file specified, opening default: {DEFAULT_FILE}");
                    hide = true;
                    file = DEFAULT_FILE.to_string();
                },
                _ => file = (&argv[1]).to_string(),
            }
            return (file, hide);
        },
        _ => {
            // catch all (only uses first 2 args excluding rpass)
            match &argv[1] as &str {
                "--hidden" => hide = true,
                _ => file = (&argv[1]).to_string(),
            }
            match &argv[2] as &str {
                "--hidden" => hide = true,
                _ => file = (&argv[2]).to_string(),
            }
            return (file, hide);
        },
    }
}

fn rhelp() {
    println!("rpass | a CLI password manager written in Rust 🦀\n");
    println!("usage: \n");
    println!("rpass <file.json> | if no file is found/provided passwords.json is created");
    println!("rpass -h, --help  | display this message");
    println!("rpass --hidden    | only display passwords when searched");
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
    println!("copy      | copy password to clipboard");
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
