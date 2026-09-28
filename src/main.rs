use std::{
    env,
    fs::{self, DirEntry, create_dir_all, read_dir, remove_dir_all, rename},
    path::{Path, PathBuf},
};

use chrono::Local;

const RAT_DIR: &str = ".rat";
const RAT_HEAD: &str = "HEAD.md";
const RAT_LANE: &str = ".LANE";
mod command {
    pub const INIT: &str = "init";
    pub const LIST: &str = "ls";
    pub const LSP: &str = "lsp";
    pub const LANE: &str = "lane";
    pub const REMOVE: &str = "rm";
    pub const ADD: &str = "add";
    pub const MOVE: &str = "mv";
}

struct Rat;

impl Rat { 
    fn get_lane(rat_path: &PathBuf) -> String {
        let lane = rat_path.join(RAT_LANE);
        fs::read_to_string(lane).unwrap()
    }
}

fn main() {
    let mut args = env::args().skip(1);
    let mut path = PathBuf::from(RAT_DIR);
    //let dir = fs::read_dir(&path) 
    match (args.next().as_deref(), path.is_dir()) {
        (Some(command::INIT), false) => {
            create_dir_all(&path);
            fs::write(path.join(RAT_LANE), "open");
            create_dir_all(path.join("open"));
            println!("Rat has been initiated");
        }
        (Some(command::LSP),true) => {
            println!("Soon...")
        }
        (Some(command::LANE), true) => {
            match args.next().as_deref() {
                None => {
                    let c_lane = fs::read_to_string(path.join(RAT_LANE)).unwrap();
                    println!("Current lane: {}", c_lane);
                }
                Some(command::LIST) => {
                    let rd = read_dir(path).unwrap();
                    for e in rd {
                        e.map(|entry| {
                            if entry.metadata().unwrap().is_dir() {
                                println!("{}", entry.file_name().to_string_lossy())
                            }
                        });
                    }
                }
                Some(command::REMOVE) => {
                    match args.next().as_deref() {
                        Some(lane) => {
                            let target = path.join(lane);
                            if let Err(e) = remove_dir_all(&target) {
                                println!("{}", e.to_string());
                                return;
                            }
                            println!("Lane `{}` removed", lane);
                        }
                        None => {
                            println!("Try doing: rat lane {} <lane>", command::REMOVE);
                        }
                    }
                }
                Some(arg) => {
                    fs::write(path.join(RAT_LANE), &arg);
                    create_dir_all(path.join(&arg));
                    println!("Current lane: {}", arg);
                }
            }
        }
        (Some(command::REMOVE), true) => {
            let title = args.collect::<Vec<_>>().join(" ");
            let target = path.join(Rat::get_lane(&path)).join(&title);
            if let Err(e) =  remove_dir_all(&target) {
                println!("{}", e.to_string());
                return;
            }
            println!("Task `{title}` removed");
        }
        (Some(command::MOVE), true) => {
            match args.next().as_deref() {
                Some(lane) => {
                    let mut target = path.join(lane);
                    if !target.is_dir() {
                        println!("Lane {} does not exist yet! Run rat lane <lane> to create one", lane);
                        return;
                    }
                    let title = args.collect::<Vec<_>>().join(" ");
                    let old_lane = Rat::get_lane(&path);
                    let mut c_lane = path.join(&old_lane);
                    target = target.join(&title);
                    c_lane = c_lane.join(&title);
                    if let Err(e) = rename(c_lane, target) {
                        println!("{}",e.to_string());
                        return;
                    } 
                    println!("Task `{title}` moved from `{old_lane}` to `{lane}`");
                }
                None => {
                    println!("How to use: rat <lane> <task-name>")
                }
            }
        }
        (Some(command::LIST), true) => {
            let lane = Rat::get_lane(&path);
            let c_path = path.join(&lane);
            let rd = read_dir(c_path).unwrap();
            for e in rd {
                e.map(|entry| {
                    println!("{}", entry.file_name().to_string_lossy());
                });
            }
        }
        (Some(command::ADD), true) => {
            let user_child = std::process::Command::new("git").args(&["config", "user.name"]).output();
            let user_email = std::process::Command::new("git").args(&["config", "user.email"]).output();
            let author = match (user_child, user_email) {
                (Err(_), Err(_)) => {
                    "null".into()
                }
                (Ok(name), Ok(email)) => {
                    let mut user_name = String::from_utf8_lossy(&name.stdout).trim().to_string();
                    let email = String::from_utf8_lossy(&email.stdout).trim().to_string();
                    //&format!("{user_name} {email}")
                    user_name.push(' ');
                    user_name.push_str(&email);
                    println!("{}", user_name);
                    user_name
                    //let w = c.wait_with_output().unwrap();
                }
                _ => "null".into()
            };
            let title = args.collect::<Vec<_>>().join(" ");
            let now = Local::now();
            let huid = now.format("%Y-%m-%d_%H-%M-%S").to_string();
            let date_format = now.format("%a %b %d %H:%M:%S %Y %z").to_string();
            //let created = now.format("%a %b %e %H:%M:%S %Y %z").to_string();
            let dir_name = format!("{} {}", title, huid);
            let lane = Rat::get_lane(&path);
            path = path.join(lane);
            path = path.join(&dir_name);

            if let Err(e) = fs::create_dir_all(&path) {
                println!("Dir failed to create {:?}", path);
                println!("{}", e.to_string());
            }
            let format = format!(
r#"Author:   {author}
Date:     {date_format}

# {title}"#
            );
            let rat_file = path.join(RAT_HEAD);
            if let Err(e) = fs::write(&rat_file, format) {
                println!("File failed to create {:?}", rat_file);
                println!("{}", e.to_string())
            }
            println!("New rat food at {}", dir_name);
            return;
        } 
        (None, true) => {
            println!("rat add <task-name> is how you use it")
        }
        (_, false) => {
            println!("Not suitable directory for rat, try running `rat init`")
        }
        _ => ()
    }
}
