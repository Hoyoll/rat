use std::{
    env,
    fs::{self, DirEntry, create_dir_all},
    path::PathBuf,
};

use chrono::Local;

const RAT_DIR: &str = ".rat";
const RAT_HEAD: &str = "HEAD.md";
const RAT_LANE: &str = ".LANE";
mod command {
    pub const INIT: &str = "init";
    //pub const SWITCH: &str = "switch";
    pub const LSP: &str = "lsp";
    pub const LANE: &str = "lane";
    pub const REMOVE: &str = "rm";
    pub const ADD: &str = "add";
}

struct Rat {
    dir: Vec<DirEntry>,
    work_dir: PathBuf,
}

impl Rat {
    fn ls(&mut self) {
        //fs::read_to_string(path)
        let rat = self.work_dir.join(RAT_DIR);
        let lane = fs::read_to_string(rat.join(RAT_LANE)).unwrap();
        let current_lane = rat.join(lane);
    }
}

fn main() {
    let mut args = env::args().skip(1);
    let mut path = PathBuf::from(RAT_DIR);
    //let dir = fs::read_dir(&path) 
    match args.next().as_deref() {
        Some(command::INIT) => {
            create_dir_all(&path);
            fs::write(path.join(RAT_LANE), "open");
            println!("Rat has been initiated");
        }
        Some(command::LSP) => {
            println!("Soon...")
        }
        Some(command::LANE) if path.is_dir() => {
            match args.next().as_deref() {
                None => {
                    let c_lane = fs::read_to_string(path.join(RAT_LANE)).unwrap();
                    println!("Current lane: {}", c_lane);
                }
                Some(arg) => {
                    fs::write(path.join(RAT_LANE), &arg);
                    println!("Current lane: {}", arg);
                }
            }
        }
        Some(command::REMOVE) if path.is_dir() => {
            match args.next().as_deref() {
                Some(command::LANE) => {}
                Some(rat) => {}
                None => {}
            }
        }
        Some(command::ADD) if path.is_dir() => {
            //if let Some(priority) = s.parse::<u32>().ok() {
            //path = path.join(lane);
            //if path.is_dir() {
            //    println!("Lane `{}` currently does not exist yet! Try running `rat lane <lane>` to create new one", lane);
            //    return; 
            //}
            //
            //
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
            let lane = fs::read_to_string(path.join(RAT_LANE)).unwrap();
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
            //    }
            println!("rat <priority> <task-name> is how you use it")
        }
        Some(_) => {
            println!("Not suitable directory for rat, try running `rat init`")
        }
        None => {
            println!("rat add <task-name> is how you use it")
        }
    }
}
