use serde_yaml;

use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};

use clap;
use clap::{Arg};
use dirs;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde::de::{value, Deserializer, Visitor, SeqAccess};


fn empty_vec() -> Vec<String> {
    return vec![];
}

fn string_or_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
    where D: Deserializer<'de>
{
    // copy-pasted from: https://github.com/serde-rs/serde/issues/889
    struct StringOrVec;

    impl<'de> Visitor<'de> for StringOrVec {
        type Value = Vec<String>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or list of strings")
        }

        fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where E: serde::de::Error
        {
            Ok(vec![s.to_owned()])
        }

        fn visit_seq<S>(self, seq: S) -> Result<Self::Value, S::Error>
            where S: SeqAccess<'de>
        {
            Deserialize::deserialize(value::SeqAccessDeserializer::new(seq))
        }
    }

    deserializer.deserialize_any(StringOrVec)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct AssociationCfg {
    #[serde(default = "empty_vec", deserialize_with = "string_or_vec")]
    ext: Vec<String>,
    #[serde(default = "empty_vec", deserialize_with = "string_or_vec")]
    mime: Vec<String>,
    #[serde(default = "empty_vec", deserialize_with = "string_or_vec")]
    mode: Vec<String>,
    #[serde(default = "empty_vec", deserialize_with = "string_or_vec")]
    multiple_files: Vec<String>,
    cmd: String,
    print: Option<String>,
}

type AssociationCfgMap = IndexMap<String, AssociationCfg>;


#[derive(PartialEq, Debug, Clone, Copy)]
enum MatchMode {
    All,
    One,
    Majority,
    Minority,
}


#[derive(PartialEq, Debug, Clone)]
struct Association {
    name: String,
    ext: Vec<String>,
    mime: Vec<String>,
    mode: Vec<String>,
    cmd: String,
    match_mode: MatchMode,
    iterate: bool,
    print: String,
}

impl Association {
    fn new(name: &String, cfg: &AssociationCfg) -> Association {
        return Association {
            mime: cfg.mime.clone(),
            name: name.clone(),
            ext: cfg.ext.clone(),
            mode: cfg.mode.clone(),
            cmd: cfg.cmd.clone(),
            print: cfg.print.clone().unwrap_or("".to_string()),
            match_mode: if cfg.multiple_files.contains(&"match-one".to_string()) {
                MatchMode::One
            } else if cfg.multiple_files.contains(&"match-all".to_string()) {
                MatchMode::All
            } else if cfg.multiple_files.contains(&"match-majority".to_string()) {
                MatchMode::Majority
            } else if cfg.multiple_files.contains(&"match-minority".to_string()) {
                MatchMode::Minority
            } else {
                MatchMode::All
            },
            iterate: cfg.multiple_files.contains(&"iterate".to_string()),
        }
    }

    fn match_file(&self, mode: &str, paths: &Vec<String>) -> Vec<String> {
        if mode == "default" {
            if self.mode.len() > 0 {
                return vec![];
            }
        } else if self.mode.len() > 0 && !self.mode.contains(&mode.to_string()) {
            return vec![];
        }

        let mut ret: Vec<String> = vec![];

        for path in paths {
            let ppath = Path::new(&path);
            let matched: bool = match ppath.extension() {
                Some(ext) => {
                    let ext = ext.to_os_string().into_string().unwrap().to_lowercase();
                    self.ext.is_empty() || self.ext.contains(&ext)
                },
                _ => false,
            };
            if matched {
                ret.push(path.to_string());
            }
        }
        match self.match_mode {
            MatchMode::One => {
                return ret;
            },
            MatchMode::All => {
                if ret.len() == paths.len() {
                    return ret;
                } else {
                    return vec![];
                }
            },
            MatchMode::Majority => {
                if ret.len() >= paths.len() / 2 {
                    return ret;
                } else {
                    return vec![];
                }
            },
            MatchMode::Minority => {
                if ret.len() >= paths.len() / 4 {
                    return ret;
                } else {
                    return vec![];
                }
            },
        }
    }

    fn run_cmd(&self, cmdv:  Vec<String>) {
        let cmd = &cmdv[0].as_str();

        let mut args: Vec<String> = vec![];
        for i in 1..cmdv.len() {
            args.push(cmdv[i].to_string());
        }

        let mut child = Command::new(cmd)
            .args(&args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("failed to execute command");

        child.wait().expect("failed to wait on command");
    }

    fn run(&self, paths: &Vec<String>) {

        if !self.iterate {
            if self.print != "" {
                println!("\n{}", self.print.replace("${files}", &paths.join(" ").as_str()));
            }

            let cmd = self.cmd
                .replace("${file}", paths[0].as_str())
                .replace("${files}", paths.join(" ").as_str());

            // FIXME: use shlex
            let cmdv: Vec<String> = cmd.split(" ").map(String::from).collect();

            self.run_cmd(cmdv);
        } else {
            for path in paths {
                if self.print != "" {
                    println!("\n{}", self.print.replace("${file}", &path.as_str()));
                }

                let cmd = self.cmd
                    .replace("${file}", path.as_str())
                    .replace("${files}", path.as_str());

                // FIXME: use shlex
                let cmdv: Vec<String> = cmd.split(" ").map(String::from).collect();

                self.run_cmd(cmdv);
            }
        }
    }
}

fn load_config() -> Vec<Association> {
    let config_path = dirs::home_dir()
        .expect("Unable to find home directory")
        .join(".nepo.yml");
    let mut associations: Vec<Association> = vec![];

    let mut file = File::open(config_path).expect("Unable to open config file");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("Unable to read config file");
    let association_cfgs: AssociationCfgMap = serde_yaml::from_str(&contents).expect("Unable to parse config file");

    for (name, asso_cfg) in &association_cfgs {
        associations.push(Association::new(name, asso_cfg));
    }

    return associations;
}

fn associate_paths(paths: &Vec<String>, mode: &str, associations: &Vec<Association>) -> (Association, Vec<String>) {
    let mut i = associations.len();

    loop {
        i = i - 1;
        if i <= 0 {
            return (associations[0].clone(), paths.clone());
        } else {
            let matches = associations[i].match_file(mode, paths);
            if matches.len() > 0 {
                return (associations[i].clone(), matches);
            }
        }
    }
}

fn main() {

    let m = clap::Command::new("nepo")
        .version("0.1.1")
        .about("Open files according to their type")
        .long_about("
nepo lets you open files with a different cli tool and
arguments according to the file extension.

nepo expects at configuration file named '~/.nepo.yml'
        ")
        .author("Frédéric van der Essen")
        .arg(
            Arg::new("debug")
                .long("debug")
                .short('d')
                .takes_value(false)
                .help("Activate debug logs")
                .required(false)
        )
        .arg(
            Arg::new("view")
                .long("view")
                .short('v')
                .takes_value(false)
                .help("Open file in view mode")
                .required(false)
        )
        .arg(
            Arg::new("edit")
                .long("edit")
                .short('e')
                .takes_value(false)
                .help("Open file in edit mode")
                .required(false)
        )
        .arg(
            Arg::new("mode")
                .long("mode")
                .short('m')
                .takes_value(true)
                .help("Open file in provided mode")
                .required(false)
        )
        .arg(
            Arg::new("filename")
                .help("The path of the file(s) to open")
                .index(1)
                .takes_value(true)
                .multiple_values(true)
                .required(true)
        )
        .after_help("")
        .get_matches();

    let associations = load_config();
    let paths: Vec<String> = m.get_many("filename")
        .expect("Please provide the path of the file you want to open.")
        .cloned()
        .collect();
    let mode = if m.is_present("mode") {
        m.value_of("mode").expect("no mode provided")
    } else if m.is_present("view") {
        "view"
    } else if m.is_present("edit") {
        "edit"
    } else {
        "default"
    };

    let (association, matched_paths) = associate_paths(&paths, mode, &associations);

    if m.is_present("debug") {
        println!("Associations:");
        for a in associations {
            println!("  {:?}", a);
        }
        println!("\nMatch:\n  {:?}", association);
    }

    association.run(&matched_paths);
}
