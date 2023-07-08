use std::{sync::Arc, io::{Write, BufReader, Read}};

use crate::{global::Global, inodes::{directory::Directory, inode::{InodeType, Inode}, metadata::Metadata, file::File}, stored::Stored};

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
        print!("\n/{}> ", match path.len() {
            0 => String::from(""),
            1 => path.last().unwrap().clone(),
            _ => format!("../{}", path.last().unwrap())
        });
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
    ("ls", ls, "Lists the contents of the current directory."),
    ("mkdir", mkdir, "Creates a new directory."),
    ("cd", cd, "Changes the current working directory."),
    ("rm", rm, "Removes a file or directory."),
    ("upload", upload, "Uploads a file to the drive"),
    ("stat", stat, "Prints metadata about a file or directory."),
    ("dbg", dbg, "Prints debug information about an object."),
    ("root", |_, _, path, cwd| { path.clear(); cwd.clear(); Ok(()) }, "Returns to root directory"),
    ("exit", |_, _, _, _| Err("SIGTERM".to_string()), "Exits the shell."),
    ("cwd", |_, _, path, _| Ok(print!("/{}", path.join("/"))), "Prints the current working directory."),
];

fn help(_global: &Arc<Global>, _args: Vec<String>, _path: &mut Vec<String>, _cwd: &mut Vec<Stored>) -> Result<(), String> {
    print!("Commands:");
    for (name, _, description) in COMMANDS {
        print!("\n  {:<10} {}", name, description);
    }
    Ok(())
}

fn dbg(global: &Arc<Global>, args: Vec<String>, _path: &mut Vec<String>, cwd: &mut Vec<Stored>) -> Result<(), String> {
    if args.len() != 1 {
        return Err("Usage: dbg <global|.|<path>>".to_string());
    }
    if args[0] == "global" {
        dbg!(global);
        Ok(())
    } else if args[0] == "." {
        let rt = Runtime::new().unwrap();
        if cwd.is_empty() {
            dbg!(global.get_root());
        } else {
            let inode: InodeType = rt.block_on(cwd.last().unwrap().get(global.clone()))?;
            dbg!(inode);
        }
        Ok(())
    } else {
        let rt = Runtime::new().unwrap();
        let dir = match cwd.last() {
            Some(cwd) => {
                let inode: InodeType = rt.block_on(cwd.get(global.clone()))?;
                match inode {
                    InodeType::Directory(dir) => dir,
                    _ => Err("Not in a directory.".to_string())?
                }
            },
            None => global.get_root()
        };
        let stored = dir.get(&args[0])?;
        let inode: InodeType = rt.block_on(stored.get(global.clone()))?;
        dbg!(inode);
        Ok(())
    }
}

fn ls(global: &Arc<Global>, _args: Vec<String>, _path: &mut Vec<String>, cwd: &mut Vec<Stored>) -> Result<(), String> {
    let rt = Runtime::new().unwrap();
    let (dir, parent) = match cwd.last() {
        Some(cwd) => {
            let inode: InodeType = rt.block_on(cwd.get(global.clone()))?;
            match inode {
                InodeType::Directory(dir) => (dir, ".."),
                _ => Err("Not in a directory.".to_string())?
            }
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
        let inode: InodeType = rt.block_on(cwd.get(global.clone()))?;
        let mut dir = match inode {
            InodeType::Directory(dir) => dir,
            _ => Err("Not in a directory.".to_string())?
        };
        rt.block_on(async {
            dir.add(global.clone(), &args[0], Directory::new().to_enum()).await
        })?;
        rt.block_on(async {
            cwd.put(global.clone(), dir.to_enum()).await
        })?;
    }
    Ok(())
}

fn cd(global: &Arc<Global>, args: Vec<String>, path: &mut Vec<String>, cwd: &mut Vec<Stored>) -> Result<(), String> {
    if args.len() != 1 {
        return Err("Usage: cd <path>".to_string());
    }
    let rt = Runtime::new().unwrap();
    let dir = match cwd.last() {
        Some(cwd) => {
            let inode: InodeType = rt.block_on(cwd.get(global.clone()))?;
            match inode {
                InodeType::Directory(dir) => dir,
                _ => Err("Not in a directory.".to_string())?
            }
        },
        None => global.get_root()
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

fn rm(global: &Arc<Global>, args: Vec<String>, _path: &mut Vec<String>, cwd: &mut Vec<Stored>) -> Result<(), String> {
    if args.len() != 1 {
        return Err("Usage: rm <name>".to_string());
    }
    if cwd.is_empty() {
        let rt = Runtime::new().unwrap();
        let mut root = global.get_root();
        rt.block_on(async {
            root.remove(global.clone(), &args[0]).await
        })?;
        global.save_root(&root);
    } else {
        let rt = Runtime::new().unwrap();
        let cwd = cwd.last_mut().unwrap();
        let inode: InodeType = rt.block_on(cwd.get(global.clone()))?;
        let mut dir = match inode {
            InodeType::Directory(dir) => dir,
            _ => Err("Not in a directory.".to_string())?
        };
        rt.block_on(async {
            dir.remove(global.clone(), &args[0]).await
        })?;
        rt.block_on(async {
            cwd.put(global.clone(), dir.to_enum()).await
        })?;
    }
    Ok(())
}

fn stat_format(metadata: &Metadata) -> String {
    let mut s = String::new();
    s.push_str(&format!("Size: {}\n", metadata.size.human()));
    s.push_str(&format!("Created: {}\n", metadata.human_created()));
    s.push_str(&format!("Modified: {}", metadata.human_modified()));
    s
}

fn stat(global: &Arc<Global>, args: Vec<String>, _path: &mut Vec<String>, cwd: &mut Vec<Stored>) -> Result<(), String> {
    if args.len() != 1 {
        return Err("Usage: stat <name|.>".to_string());
    }
    let rt = Runtime::new().unwrap();

    if args[0] == "." {
        if cwd.is_empty() {
            let metadata: Metadata = rt.block_on(async {
                let root = global.get_root();
                root.metadata().await.clone()
            });
            println!("Type: Directory");
            print!("{}", stat_format(&metadata));
        } else {
            let inode: InodeType = rt.block_on(cwd.last().unwrap().get(global.clone()))?;
            let metadata: &Metadata = rt.block_on(inode.metadata());
            println!("Type: Directory");
            print!("{}", stat_format(metadata));
        }
    } else {
        let dir = match cwd.last() {
            Some(cwd) => {
                let inode: InodeType = rt.block_on(cwd.get(global.clone()))?;
                match inode {
                    InodeType::Directory(dir) => dir,
                    _ => Err("Not in a directory.".to_string())?
                }
            },
            None => global.get_root()
        };
        let stored = dir.get(&args[0])?;
        let inode: InodeType = rt.block_on(stored.get(global.clone()))?;
        let metadata: &Metadata = rt.block_on(inode.metadata());
        match inode {
            InodeType::Directory(_) => println!("Type: Directory"),
            InodeType::File(_) => println!("Type: File")
        }
        print!("{}", stat_format(metadata));
    }

    Ok(())
}

fn upload(global: &Arc<Global>, args: Vec<String>, _path: &mut Vec<String>, cwd: &mut Vec<Stored>) -> Result<(), String> {
    if args.len() != 1 {
        return Err("Usage: upload <file>".to_string());
    }

    let file_name = args[0].clone().split('/').last().ok_or("Invalid file name.")?.to_string();
    let file = std::fs::File::open(&args[0]).map_err(|_| "Failed to open file.")?;
    let mut reader = BufReader::new(file);
    let mut data = Vec::new();

    reader.read_to_end(&mut data).map_err(|_| "Failed to read file.")?;

    print!("Read {} bytes. Uploading...", data.len());

    let rt = Runtime::new().unwrap();
    let mut dir = match cwd.last() {
        Some(cwd) => {
            let inode: InodeType = rt.block_on(cwd.get(global.clone()))?;
            match inode {
                InodeType::Directory(dir) => dir,
                _ => Err("Not in a directory.".to_string())?
            }
        },
        None => global.get_root()
    };
    let file = rt.block_on(File::create(global.clone(), data))?;
    rt.block_on(dir.add(global.clone(), &file_name, file.to_enum()))?;
    if cwd.is_empty() {
        global.save_root(&dir);
    } else {
        let cwd = cwd.last_mut().unwrap();
        rt.block_on(async {
            cwd.put(global.clone(), dir.to_enum()).await
        })?;
    }
    Ok(())
}