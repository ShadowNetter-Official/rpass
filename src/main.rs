use tabled::{Tabled, Table, settings::{Style, object::{Rows, Columns}, Remove, Color}};
use std::{fs, env, process};
use serde::{Serialize, Deserialize};
use prompted::input;
use arboard::Clipboard;

#[derive(Tabled, Debug, Serialize, Deserialize, Clone)]
struct Password {
    service: String,
    email: String,
    username: String,
    password: String,
    index: usize,
}

fn load(file: &String) -> Vec<Password> {
    let passwords = match serde_json::from_str(file) {
        Ok(p) => p,
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

fn get_content(data: &Vec<Password>) -> String {
    serde_json::to_string_pretty(data).unwrap()
}

fn save(data: &Vec<Password>, file: &String, key: &Vec<u32>) -> usize {
    let json = serde_json::to_string_pretty(data).unwrap();
    let new_data = vignere(&json, key, true);
    write_to_file(new_data, file);
    0
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
            Ok(_) => println!("copied password to clipboard"),
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
    let (file_path, hide, encrypt, encryption_key) = get_args();
    let file = load_file(&file_path);
    if encrypt {
        let key = get_key();
        let content = vignere(&file, &key, encrypt);
        write_to_file(content, &file_path);
        quit(0);
    }
    let mut clippy = Clippy::init();
    help();
    let file_content = vignere(&file, &encryption_key, encrypt);
    let mut passwords = load(&file_content);
    let mut changes: usize = 0;
    loop {
        passwords = index(passwords);
        let command = input!(">> ");
        match &command as &str {
            "new" => {
                passwords = new_password(passwords);
                changes+=1;
            },
            "list" => display_passwords(&passwords, hide),
            "save" => changes = save(&passwords, &file_path, &encryption_key),
            "quit" => quit(changes),
            "remove" => {
                passwords = remove_password(passwords);
                changes+=1;
            },
            "copy" => clippy.copy_password(&passwords),
            "clear" => clear(),
            "search" => search(passwords.clone()),
            "forcequit" => {
                clear();
                break;
            },
            "export" => {
                let export_content = get_content(&passwords);
                export(&export_content);
            },
            _ => help(),
        }
    }
}

fn get_args() -> (String, bool, bool, Vec<u32>) {
    let argv: Vec<String> = env::args().collect(); 
    let mut encrypt: bool = false;
    let mut file: String = String::new();
    match argv.len() {
        1..3 => {
            // less than 3 args
            rhelp();
            process::exit(1);
        },
        3 => {
            // 3 args (rpass <file> --encrypt)
            // or rpass --encrypt <file>
            match &argv[1] as &str {
                "--encrypt" => encrypt = true,
                _ => file = (&argv[1]).to_string(),
            }
            match &argv[2] as &str {
                "--encrypt" => encrypt = true,
                _ => file = (&argv[2]).to_string(),
            }
            return (file, false, encrypt, Vec::<u32>::new())
        },
        4 => {
            // 4 args(rpass <file> --key >key>)
            let key = convert_key(&argv[3]);
            return (argv[1].clone(), false, encrypt, key)
        },
        _ => {
            // 5 args (rpass <file> --hidden --key <key>)
            let key = convert_key(&argv[4]);
            return (argv[1].clone(), true, encrypt, key)
        }
    }
}

fn rhelp() {
    println!("rpass | a CLI password manager written in Rust 🦀\n");
    println!("usage: \n");
    println!("rpass <file.json> --key <KEY>               | open json file and decrypts it with key (file must be encrypted first)");
    println!("rpass -h, --help                            | display this message");
    println!("rpass <file.json> --hidden --key <KEY>      | only display passwords when searched");
    println!("rpass --encrypt <file.json>                 | encrypts <file.json>");
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
    println!("export    | unencrypt and export passwords to json");
}


fn clear() {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
}

fn quit(changes: usize) {
    if changes < 1 {
        process::exit(0);
    }
    println!("You have {changes} unsaved changes");
    println!("try: forcequit to quit without saving");
}

fn main() {
    rpass();
}

fn write_to_file(content: String, file: &String) {
    match fs::write(file, content) {
        Ok(_) => (),
        Err(e) => {
            println!("error writing to {file}: {e}");
            quit(0);
        },
    }
}

fn load_file(file: &String) -> String {
    let content = match fs::read_to_string(file) {
        Ok(content) => content,
        Err(e) => {
            println!("error reading {file}: {e}");
            process::exit(1);
        }
    };
    content
}

fn caesar(target: char, shift: u32, encrypt: bool) -> char {
    if !target.is_ascii_lowercase() { return target; }
    let ascii = target as u8 - b'a';
    let shifted = if encrypt {
        (ascii + (shift as u8)) % 26
    } else {
        (26 + ascii - (shift as u8 % 26)) % 26
    };
    (b'a' + shifted) as char
}

fn vignere(pass: &String, key: &Vec<u32>, encrypt: bool) -> String {
    let len = key.len();
    pass.chars()
        .enumerate()
        .map(|(i, c)| {
            let shift = key[i%len];
            caesar(c, shift, encrypt)
        })
        .collect()
}

fn get_key() -> Vec<u32> {
    let key = input!("Key (DO NOT FORGET THIS KEY) >> ");
    convert_key(&key)
}

fn convert_key(key: &String) -> Vec<u32> {
    if !key.chars().all(|c| c.is_ascii_alphabetic()) {
        println!("Key must contain only ASCII letters");
        process::exit(1);
    }
    key.to_lowercase().chars().map(|c| (c as u8 - b'a') as u32).collect()
}

fn export(content: &String) {
    // save content to new file
    let file_name = input!("File name: ");
    match fs::File::create(&file_name) {
        Ok(_) => (),
        Err(e) => {
            println!("error creating {file_name}: {e}");
            process::exit(1);
        }
    };
    write_to_file(content.to_string(), &file_name);
    println!("exported passwords unencrypted to {file_name}");
}
