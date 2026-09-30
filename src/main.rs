use std::{
    collections::{HashMap, HashSet}, env, ffi::OsString, fs::{self, DirEntry, create_dir_all, read_dir, remove_dir_all, rename, symlink_metadata}, path::{Path, PathBuf}
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
    pub const REMOVE: &str = "rm";
}

mod op {
    pub const AND: &str = ":";
    pub const OR: &str = "+";
    pub const NOT: &str = "-";
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
    pub const PATH: &str = "path";
    pub const HELP: &str = "help";
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
            create_dir_all(path.join(dir::REMOVE));
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
        (Some(command::INIT), true) => {
            create_dir_all(path.join(dir::LANE));
            create_dir_all(path.join(dir::TAG));
            create_dir_all(path.join(dir::TASK));
            create_dir_all(path.join(dir::REMOVE));
            println!("Rat has been (re)initiated");
        }
        (Some(command::LSP), true) => {
            println!("Soon...")
        }
        (Some(command::LANE), true) => {
            let lane_dir = path.join(dir::LANE);
            let mut config = Rat::get_config(&path);
            match args.next().as_deref() {
                None => {
                    println!("Current lane: {}", config.current_lane);
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
                }
                Some(command::MOVE) => {
                    if let Some(lane) = args.next() {
                        config.current_lane = lane;
                    } 
                    fs::write(path.join(RAT_CONFIG), toml::to_string(&config).unwrap());
                    println!("Current lane: {}", config.current_lane);
                }
                Some(command::LIST) => {
                    let dir = read_dir(&lane_dir).unwrap();
                    for e in dir {
                        e.map(|entry| {
                            println!("{}", entry.file_name().to_string_lossy())
                        });
                    }
                }
                Some(arg) => {
                    config.current_lane = arg.to_string();
                    //fs::write(path.join(RAT_CONFIG), toml::to_string(&config).unwrap());
                    create_dir_all(lane_dir.join(&arg));
                    println!("Created lane: {}", arg);
                }
            }
        }
        (Some(command::REMOVE), true) => {
            let title = args.collect::<Vec<_>>().join(" ");
            let target = path.join(dir::TASK).join(&title);
            let dest = path.join(dir::REMOVE).join(&title);
            if let Err(e) = rename(&target, &dest) {
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
            let title = args.collect::<Vec<_>>().join(" ");
            let dir_name = match config.collision_protection {
                true => {
                    let now = Local::now();
                    //let date_format = now.format("%a %b %d %H:%M:%S %Y %z").to_string();

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
            if let Err(e) = Rat::symlink(dest, target_lane) {
                eprintln!("File failed to create symlink for {:?}", &dir_name);
                eprintln!("{}", e.to_string());
                return;
            }
            println!("New rat food at {}", dir_name);
            return;
        }
        (Some(command::PATH), true) => {
            let mut task_path = path.join(dir::TASK);
            let title = args.collect::<Vec<_>>().join(" ");
            task_path = task_path.join(&title);
            if !task_path.is_dir() {
                return eprintln!("Task `{title}` does not exist!")
            }
            println!("{}", task_path.to_string_lossy())
        }
        (Some(command::TAG), true) => {
            match args.next().as_deref() {
                Some(command::ADD) => {
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
                        eprintln!("Task `{title}` does not exist!");
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
                }
                Some(command::LIST) => {
                    let tag_dir = path.join(dir::TAG);
                    match args.next().as_deref() {
                        Some(op::OR) => {
                            let tags = args.collect::<Vec<_>>();
                            let mut collect = HashSet::new();
                            //collect.intersection(other)
                            for tag in &tags {
                                let current_tag = tag_dir.join(tag);
                                let dir = read_dir(&current_tag).unwrap();
                                for e in dir {
                                    e.map(|entry| {
                                        let p = entry.path();
                                        if fs::metadata(&p).is_err() {
                                            fs::remove_file(&p);
                                            return;
                                        }
                                        collect.insert(entry.file_name());
                                    });
                                }
                            }
                            for task in collect {
                                println!("{}", task.to_string_lossy());
                            }
                        }
                        Some(op::AND) => {
                            let tags = args.collect::<Vec<_>>();
                            let mut collect = HashMap::<OsString, String>::new();
                            
                            for tag in &tags {
                                let current_tag = tag_dir.join(tag);
                                let dir = read_dir(&current_tag).unwrap();
                                for e in dir {
                                    e.map(|entry| {
                                        let p = entry.path();
                                        if fs::metadata(&p).is_err() {
                                            fs::remove_file(&p);
                                            return;
                                        }
                                        let name = entry.file_name();
                                        if let Some(t) = collect.get_mut(&name) {
                                            t.push_str(tag);
                                        } else {
                                            collect.insert(name, tag.into());
                                        }
                                        //collect.insert(entry, v)
                                        //collect.insert(entry.file_name());
                                    });
                                }
                            }
                            let col = tags.join("");
                            
                            for (name, str) in collect {
                                if str.as_str() == col.as_str() {
                                    println!("{}", name.to_string_lossy())
                                }
                            }
                        }// for exclusive
                        Some(op::NOT) => {
                            unimplemented!("NOT is NOT implemented yet!")
                        }// for object not having those tags
                        _ => eprintln!("Invalid operation! we only support {} and {}", op::AND, op::OR) 
                    }
                }
                _ => {
                    println!("Options: [{}, {}]", command::ADD, command::LIST)
                }
            }
            //let title =
        }
        (Some(command::HELP), false | true) => {
            println!("Basic usage: `rat`");
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
        }
    }
}
