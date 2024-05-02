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
    cmd: String,
}

type AssociationCfgMap = IndexMap<String, AssociationCfg>;


#[derive(PartialEq, Debug, Clone)]
struct Association {
    name: String,
    ext: Vec<String>,
    mime: Vec<String>,
    cmd: String,
}

impl Association {
    fn new(name: &String, cfg: &AssociationCfg) -> Association {
        return Association {
            mime: cfg.mime.clone(),
            name: name.clone(),
            ext: cfg.ext.clone(),
            cmd: cfg.cmd.clone(),
        }
    }

    fn match_file(&self, path: &str) -> bool {
        let path = Path::new(path);
        return match path.extension() {
            Some(ext) => {
                let ext = ext.to_os_string().into_string().unwrap().to_lowercase();
                return self.ext.contains(&ext);
            },
            _ => false,
        };
    }

    fn run(&self, path: &str) {
        let cmdv: Vec<&str> = self.cmd.split(" ").collect();
        let cmd = cmdv[0];
        let mut args: Vec<String> = vec![];

        let mut i = 1;

        loop {
            if i >= cmdv.len() {
                break;
            }
            if cmdv[i] == "$file" || cmdv[i] == "$files" {
                args.push(path.to_string());
            } else {
                args.push(cmdv[i].to_string());
            }
            i = i + 1;
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

fn associate_path(path: &str, associations: &Vec<Association>) -> Association {
    let mut i = associations.len();

    loop {
        i = i - 1;
        if i <= 0 {
            return associations[0].clone();
        } else if associations[i].match_file(path) {
            return associations[i].clone();
        }
    }
}

fn main() {

    let m = clap::Command::new("nepo")
        .version("0.1.0")
        .about("Open files according to their type")
        .long_about("
            Nepo lets you open files with a different
            cli tool and arguments according to the
            file and it's content types and properties.

            The configuration file is './config.yaml'
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
            Arg::new("filename")
                .help("The path of the file to open")
                .index(1)
                .required(true)
        )
        .after_help("")
        .get_matches();

    let associations = load_config();
    let path = m.value_of("filename").expect("Please provide the path of the file you want to open.");
    let association = associate_path(path, &associations);

    if m.is_present("debug") {
        println!("Associations:");
        for a in associations {
            println!("  {:?}", a);
        }
        println!("\nMatch:\n  {:?}", association);
    }

    association.run(path);
}
