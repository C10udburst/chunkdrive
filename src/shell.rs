use std::{sync::Arc, io::Write};

use crate::{global::Global, inodes::directory::Directory, stored::Stored};

use tokio::runtime::Runtime;

fn tokenize_line(line: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut token = String::new();

    let mut in_string = false;
    let mut escape = false;
    
    for c in line.chars() {
        if c == '\n' || c == '\r' {
            continue;
        } else if escape {
            token.push(c);
            escape = false;
        } else if c == '\\' {
            escape = true;
        } else if c == '"' {
            in_string = !in_string;
            token.push(c);
        } else if c == ' ' && !in_string {
            if !token.is_empty() {
                tokens.push(token);
                token = String::new();
            }
        } else {
            token.push(c);
        }
    }
    if !token.is_empty() {
        tokens.push(token);
    }
    tokens
}

pub fn shell(global: Arc<Global>) {
    print!("Welcome to the ChunkDrive {} debug shell! Type \"help\" for a list of commands.", env!("CARGO_PKG_VERSION"));

    let mut path: Vec<String> = Vec::new();
    let mut stored_cwd: Vec<Stored> = Vec::new();

    loop {
        print!("\n{}> ", path.join("/")); // TODO: make this smarter, so it doesn't print the full path if it's too long
        std::io::stdout().flush().ok();

        let mut line = String::new();
        match std::io::stdin().read_line(&mut line) {
            Ok(_) => {},
            Err(_) => continue
        }
        let tokens = tokenize_line(&line);

        if tokens.is_empty() {
            continue;
        }

        let command = tokens[0].as_str();
        let args = tokens[1..].to_vec();

        match COMMANDS.iter().find(|(name, _, _)| *name == command) {
            Some((_, func, _)) => {
                match func(&global, args, &mut path, &mut stored_cwd) {
                    Ok(_) => {},
                    Err(e) => {
                        if e == "SIGTERM" {
                            break;
                        }
                        println!("Error: {}", e)
                    }
                }
            },
            None => println!("Unknown command: {}", command)
        }
    }
}

const COMMANDS: &[(&str, fn(&Arc<Global>, Vec<String>, &mut Vec<String>, &mut Vec<Stored>) -> Result<(), String>, &str)] = &[
    ("help", help, "Prints this help message."),
    ("exit", |_, _, _, _| Err("SIGTERM".to_string()), "Exits the shell."),
    ("cwd", |_, _, path, _| Ok(print!("{}/", path.join("/"))), "Prints the current working directory."),
    ("ls", ls, "Lists the contents of the current directory."),
    ("mkdir", mkdir, "Creates a new directory."),
    ("cd", cd, "Changes the current working directory."),
];

fn help(_global: &Arc<Global>, _args: Vec<String>, _path: &mut Vec<String>, _cwd: &mut Vec<Stored>) -> Result<(), String> {
    print!("Commands:");
    for (name, _, description) in COMMANDS {
        print!("\n  {:<10} {}", name, description);
    }
    Ok(())
}

fn ls(global: &Arc<Global>, _args: Vec<String>, _path: &mut Vec<String>, cwd: &mut Vec<Stored>) -> Result<(), String> {
    let rt = Runtime::new().unwrap();
    let (dir, parent) = match cwd.last() {
        Some(cwd) => {
            (rt.block_on(cwd.get(global.clone()))?, "..")
        },
        None => (global.get_root(), ".")
    };
    
    print!("{}", parent);
    for name in dir.list() {
        print!("\n{}", name);
    }
    Ok(())
}

fn mkdir(global: &Arc<Global>, args: Vec<String>, _path: &mut Vec<String>, cwd: &mut Vec<Stored>) -> Result<(), String> {
    if args.len() != 1 {
        return Err("Usage: mkdir <name>".to_string());
    }
    if cwd.is_empty() {
        // root directory
        let rt = Runtime::new().unwrap();
        let mut root = global.get_root();
        rt.block_on(async {
            root.add(global.clone(), &args[0], Directory::new().to_enum()).await
        })?;
        global.save_root(&root);

    } else {
        let rt = Runtime::new().unwrap();
        let cwd = cwd.last_mut().unwrap();
        let mut dir: Directory = rt.block_on(cwd.get(global.clone()))?;
        rt.block_on(async {
            dir.add(global.clone(), &args[0], Directory::new().to_enum()).await
        })?;
        rt.block_on(async {
            cwd.put(global.clone(), dir).await
        })?;
    }
    Ok(())
}

fn cd(global: &Arc<Global>, args: Vec<String>, path: &mut Vec<String>, cwd: &mut Vec<Stored>) -> Result<(), String> {
    if args.len() != 1 {
        return Err("Usage: cd <path>".to_string());
    }
    let rt = Runtime::new().unwrap();
    let (dir, _) = match cwd.last() {
        Some(cwd) => {
            (rt.block_on(cwd.get(global.clone()))?, "..")
        },
        None => (global.get_root(), ".")
    };
    if args[0] == ".." {
        if !path.is_empty() {
            path.pop();
        }
        if !cwd.is_empty() {
            cwd.pop();
        }
    } else {
        let mut found = false;
        for name in dir.list() {
            if name == args[0] {
                found = true;
                break;
            }
        }
        if !found {
            return Err("No such directory.".to_string());
        }
        path.push(args[0].clone());
        cwd.push(dir.get(&args[0])?.clone());
    }
    Ok(())
}