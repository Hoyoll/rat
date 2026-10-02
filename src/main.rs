use std::{
    collections::{HashMap, HashSet},
    env,
    ffi::OsString,
    fs::{self, DirEntry, create_dir_all, read_dir, remove_dir_all, rename, symlink_metadata},
    path::{Path, PathBuf},
};

use chrono::Local;
use serde::{Deserialize, Serialize};

const RAT_DIR: &str = ".rat";
//const RAT_HEAD: &str = "HEAD.md";
const RAT_CONFIG: &str = "CONFIG.toml";

mod dir {
    pub const LANE: &str = "lane";
    pub const TAG: &str = "tag";
    pub const TASK: &str = "task";
    pub const REMOVE: &str = "rm";
}

mod op {
    pub const AND: &str = "AND";
    pub const OR: &str = "OR";
    pub const NOT: &str = "NOT";
}

mod command {
    pub const INIT: &str = "init";
    pub const LIST: &str = "ls";
    pub const LSP: &str = "lsp";
    pub const REMOVE: &str = "rm";
    pub const ADD: &str = "add";
    pub const TAG: &str = "tag";
    pub const PATH: &str = "path";
    pub const HELP: &str = "help";
}

struct Rat;

#[derive(Deserialize, Serialize)]
struct Config {
    //current_lane: String,
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
    let path = PathBuf::from(RAT_DIR);
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
                    //current_lane: "open".into(),
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
        (Some(command::LIST), true) => {
            let tasks = path.join(dir::TASK);
            let rd = read_dir(tasks).unwrap();
            for e in rd {
                e.map(|entry| {
                    println!("{}", entry.file_name().to_string_lossy());
                });
            }
        }
        (Some(command::ADD), true) => {
            let config = Rat::get_config(&path); 
            while let Some(title) = args.next() {
                let dir_name = match config.collision_protection {
                    true => {
                        let now = Local::now();
                        let huid = now.format("%Y-%m-%d_%H-%M-%S").to_string();
                        format!("{} {}", title, huid)
                    }
                    false => title.clone(),
                };
                let target = path.join(dir::TASK).join(&dir_name);
                if let Err(e) = fs::create_dir_all(&target) {
                    println!("Dir failed to create {:?}", &dir_name);
                    println!("{}", e.to_string());
                } 
                println!("New rat food at {}", dir_name); 
            }
        }
        (Some(command::PATH), true) => {
            let mut task_path = path.join(dir::TASK);
            let title = args.collect::<Vec<_>>().join(" ");
            task_path = task_path.join(&title);
            if !task_path.is_dir() {
                return eprintln!("Task `{title}` does not exist!");
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
                    //let title = args.collect::<Vec<_>>().join(" ");
                    while let Some(title) = args.next() {
                        let target_task = path.join(dir::TASK).join(&title);
                        if !target_task.is_dir() {
                            eprintln!("Task `{title}` does not exist!");
                            continue;
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
                }
                Some(command::LIST) => {
                    let tag_dir = path.join(dir::TAG);
                    match args.next().as_deref() {
                        Some(op::OR) => {
                            let tags = args.collect::<Vec<_>>();
                            let mut collect = HashSet::new();
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
                        } // for exclusive
                        Some(op::NOT) => {
                            unimplemented!("NOT is NOT implemented yet!")
                        } // for object not having those tags
                        _ => eprintln!(
                            "Invalid operation! we only support {} and {}",
                            op::AND,
                            op::OR
                        ),
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
        (Some("version"), false | true) => {
            println!("0.1")
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
