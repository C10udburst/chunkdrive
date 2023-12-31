use std::{sync::Arc, io::{Write, BufReader, Read}};
use liner::{Context, Completer};
use futures::StreamExt;
use tokio::runtime::Runtime;

use crate::{global::Global, inodes::{directory::Directory, inode::{InodeType, Inode}, metadata::Metadata, file::File}, stored::Stored};

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
        } else if c == '"' || c == '\'' || c == '`' {
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

struct ShellCompleter;

impl Completer for ShellCompleter {
    fn completions(&mut self, _start: &str) -> Vec<String> {
        let tokens = tokenize_line(_start);
        match tokens.len() {
            0 => COMMANDS.iter().map(|(name, _, _)| name.to_string()).collect(),
            1 => COMMANDS.iter().filter(|(name, _, _)| name.starts_with(_start)).map(|(name, _, _)| name.to_string()).collect(),
            _ => Vec::new()
        }
    }
}

pub fn shell(global: Arc<Global>) {
    println!("Welcome to the ChunkDrive {} debug shell! Type \"help\" for a list of commands.", env!("CARGO_PKG_VERSION"));

    let mut path: Vec<String> = Vec::new();
    let mut stored_cwd: Vec<Stored> = Vec::new();
    let mut clipboard: Option<Stored> = None;
    let mut context = Context::new();

    loop {
        let prompt = format!("{}/{}# ",
            match clipboard {
                Some(_) => "📋 ",
                None => ""
            },
            match path.len() {
                0 => String::from(""),
                1 => path.last().unwrap().clone(),
                _ => format!("../{}", path.last().unwrap())
            }
        );
    
        let line = match context.read_line(&prompt, None, &mut ShellCompleter) {
            Ok(line) => line,
            Err(_) => break
        };
        let tokens = tokenize_line(&line);

        if tokens.is_empty() {
            continue;
        }

        let command = tokens[0].as_str();
        let args = tokens[1..].to_vec();

        match COMMANDS.iter().find(|(name, _, _)| *name == command) {
            Some((_, func, _)) => {
                match func(&global, args, &mut path, &mut stored_cwd, &mut clipboard) {
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

type Command = (&'static str, fn(&Arc<Global>, Vec<String>, &mut Vec<String>, &mut Vec<Stored>, &mut Option<Stored>) -> Result<(), String>, &'static str);

const COMMANDS: &[Command] = &[
    ("help",   help, "Prints this help message."),
    ("exit",   exit, "Exits the shell."),
    ("ls",     ls, "Lists the contents of the current directory."),
    ("mkdir",  mkdir, "Creates a new directory."),
    ("cd",     cd, "Changes the current working directory."),
    ("rm",     rm, "Removes a file or directory."),
    ("cut",    cut, "Cuts a file or directory."),
    ("paste",  paste, "Pastes a file or directory."),
    ("up",     upload, "Uploads a file to the drive"),
    ("down",   download, "Downloads a file from the drive."),
    ("stat",   stat, "Prints metadata about a file or directory."),
    ("lsbk",   bucket_list, "Lists all buckets."),
    ("bktest", bucket_test, "Tests a bucket."),
    ("dbg",    dbg, "Prints debug information about an object."),
    ("root",   |_, _, path, cwd, _| { path.clear(); cwd.clear(); Ok(()) }, "Returns to the root directory"),
    ("cwd",    |_, _, path, _, _| Ok(println!("/{}", path.join("/"))), "Prints the current working directory."),
];

fn help(_global: &Arc<Global>, _args: Vec<String>, _path: &mut Vec<String>, _cwd: &mut Vec<Stored>, _clipboard: &mut Option<Stored>) -> Result<(), String> {
    println!("Commands:");
    for (name, _, description) in COMMANDS {
        println!("  {:<10} {}", name, description);
    }
    Ok(())
}

fn dbg(global: &Arc<Global>, args: Vec<String>, _path: &mut Vec<String>, cwd: &mut Vec<Stored>, _clipboard: &mut Option<Stored>) -> Result<(), String> {
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

fn ls(global: &Arc<Global>, _args: Vec<String>, _path: &mut Vec<String>, cwd: &mut Vec<Stored>, _clipboard: &mut Option<Stored>) -> Result<(), String> {
    let rt = Runtime::new().unwrap();
    let dir = match cwd.last() {
        Some(cwd) => {
            let inode: InodeType = rt.block_on(cwd.get(global.clone()))?;
            match inode {
                InodeType::Directory(dir) => {
                    println!("..");
                    dir
                }
                _ => Err("Not in a directory.".to_string())?
            }
        },
        None => global.get_root()
    };
    
    for name in dir.list() {
        println!("{}", name);
    }
    Ok(())
}

fn mkdir(global: &Arc<Global>, args: Vec<String>, _path: &mut Vec<String>, cwd: &mut Vec<Stored>, _clipboard: &mut Option<Stored>) -> Result<(), String> {
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

fn cd(global: &Arc<Global>, args: Vec<String>, path: &mut Vec<String>, cwd: &mut Vec<Stored>, _clipboard: &mut Option<Stored>) -> Result<(), String> {
    if args.len() != 1 {
        return Err("Usage: cd <path>".to_string());
    }

    if args[0] == ".." {
        if !path.is_empty() {
            path.pop();
        }
        if !cwd.is_empty() {
            cwd.pop();
        }
        return Ok(());
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
    Ok(())
}

fn rm(global: &Arc<Global>, args: Vec<String>, _path: &mut Vec<String>, cwd: &mut Vec<Stored>, _clipboard: &mut Option<Stored>) -> Result<(), String> {
    if args.len() != 1 {
        return Err("Usage: rm <name>".to_string());
    }
    if cwd.is_empty() {
        let rt = Runtime::new().unwrap();
        let mut root = global.get_root();
        let err = rt.block_on(async {
            root.remove(global.clone(), &args[0]).await
        });
        global.save_root(&root);
        err?;
    } else {
        let rt = Runtime::new().unwrap();
        let cwd = cwd.last_mut().unwrap();
        let inode: InodeType = rt.block_on(cwd.get(global.clone()))?;
        let mut dir = match inode {
            InodeType::Directory(dir) => dir,
            _ => Err("Not in a directory.".to_string())?
        };
        let err = rt.block_on(async {
            dir.remove(global.clone(), &args[0]).await
        });
        rt.block_on(async {
            cwd.put(global.clone(), dir.to_enum()).await
        })?;
        err?;
    }
    Ok(())
}

fn cut(global: &Arc<Global>, args: Vec<String>, _path: &mut Vec<String>, cwd: &mut Vec<Stored>, clipboard: &mut Option<Stored>) -> Result<(), String> {
    if args.len() != 1 {
        return Err("Usage: cut <name>".to_string());
    }
    if clipboard.is_some() {
        return Err("Clipboard is not empty.".to_string());
    }
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
    let stored = dir.unlink(&args[0])?;
    if cwd.is_empty() {
        global.save_root(&dir);
    } else {
        let cwd = cwd.last_mut().unwrap();
        rt.block_on(async {
            cwd.put(global.clone(), dir.to_enum()).await
        })?;
    }
    let _ = clipboard.insert(stored);
    Ok(())
}

fn paste(global: &Arc<Global>, args: Vec<String>, _path: &mut Vec<String>, cwd: &mut Vec<Stored>, clipboard: &mut Option<Stored>) -> Result<(), String> {
    if args.len() != 1 {
        return Err("Usage: cut <name>".to_string());
    }
    if clipboard.is_none() {
        return Err("Clipboard is empty.".to_string());
    }
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

    let stored = clipboard.take().unwrap();
    dir.put(&args[0], stored)?;

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

fn exit(_global: &Arc<Global>, _args: Vec<String>, _path: &mut Vec<String>, _cwd: &mut Vec<Stored>, clipboard: &mut Option<Stored>) -> Result<(), String> {
    if clipboard.is_some() {
        return Err("Clipboard is not empty. Paste it somewhere first.".to_string());
    }

    Err("SIGTERM".to_string())
}

fn stat_format(metadata: &Metadata) -> String {
    let mut s = String::new();
    s.push_str(&format!("Size: {}\n", metadata.size.human()));
    s.push_str(&format!("Created: {}\n", metadata.human_created()));
    s.push_str(&format!("Modified: {}", metadata.human_modified()));
    s
}

fn stat(global: &Arc<Global>, args: Vec<String>, _path: &mut Vec<String>, cwd: &mut Vec<Stored>, _clipboard: &mut Option<Stored>) -> Result<(), String> {
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
            println!("{}", stat_format(&metadata));
        } else {
            let inode: InodeType = rt.block_on(cwd.last().unwrap().get(global.clone()))?;
            let metadata: &Metadata = rt.block_on(inode.metadata());
            println!("Type: Directory");
            println!("{}", stat_format(metadata));
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
        println!("{}", stat_format(metadata));
    }

    Ok(())
}

fn upload(global: &Arc<Global>, args: Vec<String>, _path: &mut Vec<String>, cwd: &mut Vec<Stored>, _clipboard: &mut Option<Stored>) -> Result<(), String> {
    if args.len() != 1 {
        return Err("Usage: up <file>".to_string());
    }

    let file_name = args[0].clone().split('/').last().ok_or("Invalid file name.")?.to_string();
    let file = std::fs::File::open(&args[0]).map_err(|_| "Failed to open file.")?;
    let mut reader = BufReader::new(file);
    let mut data = Vec::new();

    reader.read_to_end(&mut data).map_err(|_| "Failed to read file.")?;

    println!("Read {} bytes. Uploading...", data.len());

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

    println!("Uploaded to {}.", file_name);

    Ok(())
}

fn download(global: &Arc<Global>, args: Vec<String>, _path: &mut Vec<String>, cwd: &mut Vec<Stored>, _clipboard: &mut Option<Stored>) -> Result<(), String> {
    if args.len() != 2 {
        return Err("Usage: down <from> <to>".to_string());
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

    let stored = dir.get(&args[0])?;
    let inode: InodeType = rt.block_on(stored.get(global.clone()))?;
    let file = match inode {
        InodeType::File(file) => file,
        _ => Err("Not a file.".to_string())?
    };
    let metadata = rt.block_on(file.metadata());
    println!("Downloading {}...", metadata.size.human());
    let mut buf_writer = std::io::BufWriter::new(std::fs::File::create(&args[1]).map_err(|_| "Failed to create file.")?);
    let mut stream = file.get(global.clone());
    while let Some(chunk) = rt.block_on(stream.next()) {
        let slice = chunk.map_err(|_| "Failed to read file.")?;
        buf_writer.write_all(&slice).map_err(|_| "Failed to write file.")?;
    }
    println!("Downloaded to {}.", args[1]);

    Ok(())
}

fn bucket_list(global: &Arc<Global>, _args: Vec<String>, _path: &mut Vec<String>, _cwd: &mut Vec<Stored>, _clipboard: &mut Option<Stored>) -> Result<(), String> {
    println!("  {:<20} {:<20} {:<20} {}" , "Name", "Source", "Encryption", "Max block size");
    for bucket in global.list_buckets() {
        let b_type = match global.get_bucket(bucket) {
            Some(bucket) => bucket.human_readable(),
            None => "Missing?".to_string()
        };
        println!("  {:<20} {}", bucket, b_type);
    }
    Ok(())
}

fn bucket_test(global: &Arc<Global>, args: Vec<String>, _path: &mut Vec<String>, _cwd: &mut Vec<Stored>, _clipboard: &mut Option<Stored>) -> Result<(), String> {
    if args.len() != 1 {
        return Err("Usage: bktest <name>".to_string());
    }
    let bucket = match global.get_bucket(&args[0]) {
        Some(bucket) => bucket,
        None => Err("No such bucket.".to_string())?
    };
    
    let block = vec![0; bucket.max_size()];
    
    let rt = Runtime::new().unwrap();
    let descriptor = rt.block_on(bucket.create())?;
    println!("Created descriptor: {:?}", descriptor);

    rt.block_on(bucket.put(&descriptor, block.clone()))?;
    println!("Put data of size {}.", block.len());

    let retrieved = rt.block_on(bucket.get(&descriptor))?;
    println!("Retrieved data of size {}.", retrieved.len());

    rt.block_on(bucket.delete(&descriptor))?;
    println!("Deleted data.");

    if block != retrieved {
        return Err("Data mismatch.".to_string());
    } else {
        println!("Data matches.");
    }

    let recieved2 = rt.block_on(bucket.get(&descriptor));
    if recieved2.is_ok() {
        return Err("Data still exists.".to_string());
    }
    println!("Deleted data was not found.");

    println!("OK.");

    Ok(())
}