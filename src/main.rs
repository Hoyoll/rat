use std::{
    env,
    fs::{self, DirEntry, create_dir_all, read_dir, remove_dir_all, rename},
    path::{Path, PathBuf},
};

use chrono::Local;
use serde::{Deserialize, Serialize};

const RAT_DIR: &str = ".rat";
const RAT_HEAD: &str = "HEAD.md";
const RAT_CONFIG: &str = "CONFIG.toml";

mod dir {
    pub const LANE: &str = "lane";
    pub const TAG: &str = "tag";
    pub const TASK: &str = "task";
}

mod command {
    pub const INIT: &str = "init";
    pub const LIST: &str = "ls";
    pub const LSP: &str = "lsp";
    pub const LANE: &str = "lane";
    pub const REMOVE: &str = "rm";
    pub const ADD: &str = "add";
    pub const MOVE: &str = "mv";
    pub const TAG: &str = "tag";
}

struct Rat;

#[derive(Deserialize, Serialize)]
struct Config {
    current_lane: String,
    collision_protection: bool,
}

impl Rat {
    fn symlink<P: AsRef<Path>, Q: AsRef<Path>>(original: P, link: Q) -> std::io::Result<()> {
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(original, link)
        }

        #[cfg(windows)]
        {
            //let src = original.as_ref();
            //if src.is_dir() {
            std::os::windows::fs::symlink_dir(original, link)
            //} else {
            //  std::os::windows::fs::symlink_file(src, link)
            //}
        }
    }
    fn get_config(rat_path: &PathBuf) -> Config {
        let config_path = rat_path.join(RAT_CONFIG);
        let toml_file = fs::read_to_string(&config_path).unwrap();
        toml::from_str::<Config>(&toml_file).unwrap()
    }
}

fn main() {
    let mut args = env::args().skip(1);
    let mut path = PathBuf::from(RAT_DIR);
    //let dir = fs::read_dir(&path)
    match (args.next().as_deref(), path.is_dir()) {
        (Some(command::INIT), false) => {
            create_dir_all(&path);
            create_dir_all(path.join(dir::LANE));
            create_dir_all(path.join(dir::TAG));
            create_dir_all(path.join(dir::TASK));
            fs::write(
                path.join(RAT_CONFIG),
                toml::to_string(&Config {
                    current_lane: "open".into(),
                    collision_protection: false,
                })
                .unwrap(),
            );
            create_dir_all(path.join(dir::LANE).join("open"));
            println!("Rat has been initiated");
        }
        (Some(command::LSP), true) => {
            println!("Soon...")
        }
        (Some(command::LANE), true) => {
            let lane_dir = path.join(dir::LANE);
            let mut config = Rat::get_config(&path);
            match args.next().as_deref() {
                None => {
                    //let c_lane = fs::read_to_string(path.join(RAT_LANE)).unwrap();
                    println!("Current lane: {}", config.current_lane);
                }
                Some(command::LIST) => {
                    let rd = read_dir(lane_dir).unwrap();
                    for e in rd {
                        e.map(|entry| {
                            //if entry.metadata().unwrap().is_dir() {
                            println!("{}", entry.file_name().to_string_lossy())
                            //}
                        });
                    }
                }
                Some(command::REMOVE) => match args.next().as_deref() {
                    Some(lane) => {
                        let target = lane_dir.join(lane);
                        if let Err(e) = remove_dir_all(&target) {
                            println!("{}", e.to_string());
                            return;
                        }
                        println!("Lane `{}` removed", lane);
                    }
                    None => {
                        println!("Try doing: rat lane {} <lane>", command::REMOVE);
                    }
                },
                Some(arg) => {
                    config.current_lane = arg.to_string();
                    fs::write(path.join(RAT_CONFIG), toml::to_string(&config).unwrap());
                    create_dir_all(lane_dir.join(&arg));
                    println!("Current lane: {}", arg);
                }
            }
        }
        (Some(command::REMOVE), true) => {
            let title = args.collect::<Vec<_>>().join(" ");
            let target = path.join(dir::TASK).join(&title);
            if let Err(e) = remove_dir_all(&target) {
                println!("{}", e.to_string());
                return;
            }
            println!("Task `{title}` removed");
        }
        (Some(command::MOVE), true) => {
            let lane_dir = path.join(dir::LANE);
            let config = Rat::get_config(&path);
            match args.next().as_deref() {
                Some(lane) => {
                    let mut target = lane_dir.join(lane);
                    if !target.is_dir() {
                        println!(
                            "Lane {} does not exist yet! Run rat lane <lane> to create one",
                            lane
                        );
                        return;
                    }
                    let title = args.collect::<Vec<_>>().join(" ");
                    //let old_lane = Rat::get_lane(&path);
                    let mut c_lane = lane_dir.join(&config.current_lane);
                    target = target.join(&title);
                    c_lane = c_lane.join(&title);
                    if let Err(e) = rename(c_lane, target) {
                        println!("{}", e.to_string());
                        return;
                    }
                    println!(
                        "Task `{title}` moved from `{}` to `{lane}`",
                        config.current_lane
                    );
                }
                None => {
                    println!("How to use: rat <lane> <task-name>")
                }
            }
        }
        (Some(command::LIST), true) => {
            let config = Rat::get_config(&path);
            let lane = path.join(dir::LANE).join(&config.current_lane);
            //let c_path = path.join(&lane);
            let rd = read_dir(lane).unwrap();
            for e in rd {
                e.map(|entry| {
                    println!("{}", entry.file_name().to_string_lossy());
                });
            }
        }
        (Some(command::ADD), true) => {
            let config = Rat::get_config(&path);
            let user_child = std::process::Command::new("git")
                .args(&["config", "user.name"])
                .output();
            let user_email = std::process::Command::new("git")
                .args(&["config", "user.email"])
                .output();
            let author = match (user_child, user_email) {
                (Err(_), Err(_)) => "null".into(),
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
                _ => "null".into(),
            };
            let title = args.collect::<Vec<_>>().join(" ");
            let now = Local::now();
            let date_format = now.format("%a %b %d %H:%M:%S %Y %z").to_string();
            let dir_name = match config.collision_protection {
                true => {
                    let huid = now.format("%Y-%m-%d_%H-%M-%S").to_string();
                    //let created = now.format("%a %b %e %H:%M:%S %Y %z").to_string();
                    format!("{} {}", title, huid)
                }
                false => title.clone(),
            };
            let dest = PathBuf::from("..")
                .join("..")
                .join(dir::TASK)
                .join(&dir_name);
            let target_lane = path
                .join(dir::LANE)
                .join(&config.current_lane)
                .join(&dir_name);
            let target = path.join(dir::TASK).join(&dir_name);
            if let Err(e) = fs::create_dir_all(&target) {
                println!("Dir failed to create {:?}", &dir_name);
                println!("{}", e.to_string());
            }
            let format = format!(
                r#"Author:   {author}
Date:     {date_format}

# {title}"#
            );
            let rat_file = target.join(RAT_HEAD);
            if let Err(e) = fs::write(&rat_file, format) {
                println!("File failed to create {:?}", rat_file);
                println!("{}", e.to_string())
            }
            if let Err(e) = Rat::symlink(dest, target_lane) {
                println!("File failed to create symlink for {:?}", &dir_name);
                println!("{}", e.to_string());
                return;
            }
            println!("New rat food at {}", dir_name);
            return;
        }
        (Some(command::TAG), true) => {
            let mut tags = Vec::new();
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "+" => {
                        break;
                    }
                    _ => {
                        tags.push(arg);
                    }
                }
            }
            //println!("here?");
            let title = args.collect::<Vec<_>>().join(" ");
            let target_task = path.join(dir::TASK).join(&title);
            if !target_task.is_dir() {
                println!("Task `{title}` does not exist!");
                return;
            }
            for tag in &tags {
                let tag_dir = path.join(dir::TAG).join(&tag);
                if let Err(e) = create_dir_all(&tag_dir) {
                    println!("Cannot create tag `{tag}`");
                    println!("{}", e.to_string());
                    continue;
                }
                //.join(&dir_name);
                let dest = PathBuf::from("..").join("..").join(dir::TASK).join(&title);
                if let Err(e) = Rat::symlink(&dest, tag_dir.join(&title)) {
                    println!("File failed to create symlink for {:?}", &title);
                    println!("{}", e.to_string());
                    continue;
                }
            }
            println!("Tags {:?} registered for {title}", tags);
            //let title =
        }
        (None, true) => {
            println!("rat add <task-name> is how you use it")
        }
        (_, false) => {
            println!("Not suitable directory for rat, try running `rat init`")
        }
        _ => {
            println!("rat add <task-name> is how you use it")
        
        //    println!()
        },
    }
}
