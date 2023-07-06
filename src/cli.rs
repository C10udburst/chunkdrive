use std::{io::Write, pin::Pin, future::Future, sync::{Arc, Mutex}};
use crate::{global::Global, inodes::{folder::Folder, inode::INodeType}};

type BoxedFuture<T = ()> = Pin<Box<dyn Future<Output = T>>>;

type Command = fn(Arc<Mutex<Global>>, Vec<String>, &mut Vec<(String, Folder)>) -> BoxedFuture<Result<(), String>>;

const COMMANDS: [(&str, Command, &str); 5] = [
    ("help", help, "Print this help message."),
    ("exit", exit, "Exit the shell."),
    ("cwd", cwd, "Print the current working directory."),
    ("ls", ls, "List the contents of the current folder."),
    ("mkdir", mkdir, "Create a new folder in the current folder."),
];

pub async fn shell(global: Global) {
    println!("ChunkDrive {} debug shell.", env!("CARGO_PKG_VERSION"));
    println!("Type 'help' for help.");
    let mut buf: String;
    let stdin = std::io::stdin();
    let mut path = Vec::new();
    let arc_mut = Arc::new(Mutex::new(global));
    loop {
        buf = String::new();
        print!("\n> ");
        std::io::stdout().flush().unwrap();
        stdin.read_line(&mut buf).unwrap();
        let tokens = tokenize(buf);
        if tokens.len() == 0 {
            continue;
        }
        let cmd = tokens[0].clone();
        let args = tokens[1..].to_vec();
        match COMMANDS.iter().find(|(name, _, _)| *name == cmd) {
            Some((_, command, _)) => {
                let result = command(arc_mut.clone(), args, &mut path).await;
                match result {
                    Ok(_) => {}
                    Err(e) => {
                        if e == "SIGTERM" {
                            break;
                        }
                        println!("Error: {}", e);
                    }
                }
            }
            None => println!("Unknown command: {}", cmd),
        }
    }
}

fn tokenize(cmd: String) -> Vec<String> {
    let mut tokens: Vec<String> = Vec::new();
    let mut token: String = String::new();
    let mut in_string: bool = false;
    let mut escape: bool = false;
    if cmd.len() == 0 {
        return tokens;
    }
    for c in cmd.strip_suffix('\n').unwrap().chars() {
        if escape {
            token.push(c);
            escape = false;
        } else if c == '\\' {
            escape = true;
        } else if c == '"' {
            in_string = !in_string;
        } else if c == ' ' && !in_string {
            if token.len() > 0 {
                tokens.push(token);
                token = String::new();
            }
        } else {
            token.push(c);
        }
    }
    if token.len() > 0 {
        tokens.push(token);
    }
    tokens
}

fn help(_: Arc<Mutex<Global>>, _: Vec<String>, _: &mut Vec<(String, Folder)>) -> BoxedFuture<Result<(), String>> {
    Box::pin(async {
        println!("Commands:");
        for (name, _, description) in COMMANDS.iter() {
            println!("  {}\t-\t{}", name, description);
        }
        Ok(())
    })
}

fn exit(_: Arc<Mutex<Global>>, _: Vec<String>, _: &mut Vec<(String, Folder)>) -> BoxedFuture<Result<(), String>> {
    Box::pin(async {
        Err("SIGTERM".to_string())
    })
}

fn cwd(_: Arc<Mutex<Global>>, _: Vec<String>, path: &mut Vec<(String, Folder)>) -> BoxedFuture<Result<(), String>> {
    let names  = path.iter().map(|(name, _)| name.clone()).collect::<Vec<String>>();
    Box::pin(async move {
        print!("/");
        for name in names {
            print!("{}/", name);
        }
        println!();
        Ok(())
    })
}

fn ls(global: Arc<Mutex<Global>>, _: Vec<String>, path: &mut Vec<(String, Folder)>) -> BoxedFuture<Result<(), String>> {
    if path.len() == 0 {
        let global_mut = global.lock().unwrap();
        let root = global_mut.get_root().lock().unwrap();
        let children = root.list();
        for child in children {
            println!("{}", child);
        }
        Box::pin(async move {
            Ok(())
        })
    } else {
        let folder = path.last().unwrap().1.clone();
        Box::pin(async move {
            let mut children = folder.list();
            children.sort();
            for child in children {
                println!("{}", child);
            }
            Ok(())
        })
    }
}

fn mkdir(global: Arc<Mutex<Global>>, args: Vec<String>, path: &mut Vec<(String, Folder)>) -> BoxedFuture<Result<(), String>> {
    if args.len() == 0 {
        return Box::pin(async {
            Err("No folder name specified.".to_string())
        });
    }
    if path.len() == 0 {
        return Box::pin(async move {
            let global_mut = global.as_ref().lock().unwrap();
            let mut root = global_mut.get_root().lock().unwrap();
            let name = args[0].clone();
            let folder = Folder::create().map_err(|e| format!("Failed to create folder: {}", e))?;
            root.add(&global_mut, &name, folder.to_enum()).await.map_err(|e| format!("Failed to add folder: {}", e))?;
            global_mut.root_updated();
            Ok(())
        })
    } else if path.len() == 1 {
        unimplemented!()
    } else {
        let mut parent = path.last().unwrap().1.clone();
        let parent_name = path.last().unwrap().0.clone();
        let mut parent_of_parent = path[path.len() - 2].1.clone();
        let name = args[0].clone();
        Box::pin(async move {
            let folder = Folder::create().map_err(|e| format!("Failed to create folder: {}", e))?;
            let global_mut = global.lock().unwrap();
            parent.add(&global_mut, &name, folder.to_enum()).await.map_err(|e| format!("Failed to add folder: {}", e))?;
            parent_of_parent.update(&global_mut, &parent_name, &parent.to_enum()).await.map_err(|e| format!("Failed to update parent folder: {}", e))?;
            Ok(())
        })
    }
}