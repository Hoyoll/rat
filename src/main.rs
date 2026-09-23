use std::{env, fs, path::PathBuf};

use chrono::Local;

const RAT_DIR: &str = ".rat";
const RAT_HEAD: &str = "HEAD.toml";

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct Rat<'rat> {
    title: &'rat str,
    description: &'rat str,
    references: [();0],
}

fn main() {
    let mut args = env::args().skip(1);
    let mut path = PathBuf::from(RAT_DIR);
    match args.next().as_deref() {
        Some(s) => {
            if let Some(priority) = s.parse::<u32>().ok() {
                let title = args.collect::<Vec<_>>().join(" ");
                let now = Local::now();
                let huid = now.format("%Y-%m-%d_%H-%M-%S").to_string();
                //let created = now.format("%a %b %e %H:%M:%S %Y %z").to_string();
                let dir_name = format!("{}_{}_{}",priority, title, huid);
                path = path.join(&dir_name);

                if let Err(e) = fs::create_dir_all(&path) {
                    println!("Dir failed to create {:?}", path);
                    println!("{}", e.to_string());
                }
                let rat_file = path.join(RAT_HEAD);
                if let Err(e) = fs::write(
                    &rat_file,
                    toml::to_string_pretty(&Rat {
                        title: &title,
                        description: "",
                        references: [],
                    })
                    .unwrap(),
                ) {
                    println!("File failed to create {:?}", rat_file);
                    println!("{}", e.to_string())
                }
                println!("New rat food at {}", dir_name);
                return;
            }
            println!("rat <priority> <task-name> is how you use it")
        }
        None => {
            println!("rat <priority> <task-name> is how you use it")
        }
    }
}
