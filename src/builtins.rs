use std::{collections::HashMap, env, path::Path};

use crate::{source, CommandStatus, is_debug};
pub type Builtin = fn(
    args: &Vec<String>,
    environment: &mut HashMap<String, String>,
    aliases: &mut HashMap<String, String>,
) -> CommandStatus;

fn cd(
    args: &Vec<String>,
    _: &mut HashMap<String, String>,
    __: &mut HashMap<String, String>,
) -> CommandStatus {
    let new_dir = args.get(0).map_or("/", |x| &*x);
    let root = Path::new(new_dir);
    if let Err(e) = env::set_current_dir(&root) {
        return CommandStatus::Message(e.to_string(), 1);
    }

    return CommandStatus::NORMAL;
}
fn export_or_alias(
    args: &Vec<String>,
    map: &mut HashMap<String, String>,
    name: String,
    delimiter: &str,
) -> CommandStatus {
    if delimiter.is_empty() {}
    let full_arg = args.join(" ");
    let full_arg_trim = full_arg.trim();

    if full_arg_trim.is_empty() {
        return CommandStatus::Message(format!("{}: {:#?}", name, map), 0);
    }
    if full_arg.contains(delimiter) || delimiter.is_empty() {
        let key: String;
        let value: String;
        if delimiter.is_empty() {
            let (k, v) = args.split_first().unwrap();
            key = k.to_string();
            value = v.join(" ");
        } else {
            let (k, v) = full_arg_trim.split_once(delimiter).unwrap();
            key = k.to_string();
            value = v.to_string();
        }
        map.insert(key.trim().to_string(), value.trim().to_string());
        if name == "export" {
            env::set_var(key.trim().to_string(), value.trim().to_string());
        }
    } else if map.contains_key(full_arg_trim) {
        return CommandStatus::Message(map.get(full_arg_trim).unwrap().to_string(), 0);
    }
    return CommandStatus::NORMAL;
}

fn export(
    args: &Vec<String>,
    environment: &mut HashMap<String, String>,
    __: &mut HashMap<String, String>,
) -> CommandStatus {
    return export_or_alias(args, environment, "export".to_string(), "=");
}
fn alias_bash_syntax(
    args: &Vec<String>,
    _: &mut HashMap<String, String>,
    aliases: &mut HashMap<String, String>,
) -> CommandStatus {
    return export_or_alias(args, aliases, "alias".to_string(), "=");
}
fn unalias(
    args: &Vec<String>,
    _: &mut HashMap<String, String>,
    aliases: &mut HashMap<String, String>,
)-> CommandStatus {
    for a in args {
        if aliases.contains_key(a) {
            aliases.remove(a);
        }
    };CommandStatus::NORMAL
}
fn unset(
    args: &Vec<String>,
    environment: &mut HashMap<String, String>,
    _: &mut HashMap<String, String>,
)  -> CommandStatus{
    for a in args {
        if environment.contains_key(a) {
            if is_debug(){
            println!("environment::unset::{},",a);
            }
            environment.remove(a);
            env::remove_var(a);
        }
    };CommandStatus::NORMAL
}

fn exit(
    args: &Vec<String>,
    _: &mut HashMap<String, String>,
    __: &mut HashMap<String, String>,
) -> CommandStatus {
    let mut code = 0;
    if !args.is_empty() {
        let code_r = args[0].as_str().parse();
        match code_r {
            Ok(c) => code = c,
            Err(_) => code = 1,
        }
    }
    return CommandStatus::EXIT(code);
}
fn source_command(
    args: &Vec<String>,
    environment: &mut HashMap<String, String>,
    aliases: &mut HashMap<String, String>,
) -> CommandStatus {
    let files: Vec<String> = args
        .iter()
        .filter(|p| Path::new(p).is_file())
        .map(|s| s.to_owned())
        .collect();

    source(files, environment, aliases)
}

pub fn build_map() -> HashMap<String, Builtin> {
    let mut builtins_map: HashMap<String, Builtin> = HashMap::new();
    builtins_map.insert("cd".to_string(), cd);
    builtins_map.insert("export".to_string(), export);
    builtins_map.insert("alias".to_string(), alias_bash_syntax);
    builtins_map.insert("exit".to_string(), exit);
    builtins_map.insert("source".to_string(), source_command);
    builtins_map.insert("unalias".to_string(), unalias);
    builtins_map.insert("unset".to_string(), unset);

    return builtins_map;
}
