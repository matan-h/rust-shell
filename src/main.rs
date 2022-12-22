use rustyline::completion::FilenameCompleter;
use rustyline::highlight::MatchingBracketHighlighter;
use rustyline::hint::HistoryHinter;
use rustyline::validate::MatchingBracketValidator;
use rustyline::{error::ReadlineError, Config};
use rustyline::{Cmd, CompletionType, EditMode, Editor, KeyEvent};
use shlex::Shlex;
mod lineutils;

use std::{
    collections::HashMap,
    env,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
    process::{Child, Command, Stdio},
};
// enum command_status{
//     EXIT=-1,
//     NORMAL=0,
//     ERROR=1,
// }

fn handle_command(
    raw_command: String,
    aliases: &mut HashMap<String, String>,
    environment: &mut HashMap<String, String>,
) -> i32 {
    let mut last_code_err = 0;
    let env2 = environment.clone();
    let get_env2 = |name: &str| -> Option<&str> { env2.get(name).map(|x| x.as_str()) };

    // must be peekable so we know when we are on the last command
    let command = raw_command.split("#").into_iter().next().unwrap();
    // let checker =
    // println!("checker '{}'",checker);
    if command.trim().is_empty() || command.trim().starts_with("#") {
        return 0;
    }
    let mut shl = Shlex::new(&command);
    let parts: Vec<String> = shl.by_ref().collect();
    // if shl.had_error { None } else { Some(res) }

    // let parts_err = shlex::split(&command);//.expect("shlex split not working");
    if shl.had_error {
        println!("syntax error '{}'", command);
        return 1;
    }

    let mut commands = parts.split(|n| n == "|").peekable();
    // let mut commands = command.trim().split(" | ").peekable();

    let mut previous_command = None;
    while let Some(parts) = commands.next() {
        // let args = parts.split_last().unwrap();
        // if parts.len()>1 // TODO
        let mut raw_args = parts.split_first().unwrap().1.to_vec();
        let mut raw_args_alias: Vec<String>;

        // if (command.is_empty()){continue;}

        // let command = parts.next().unwrap();
        let mut command = parts.first().unwrap().as_str();
        // println!("%'args:{:#?},command:{:#?}'%",args,command);
        // let command = parts.first().unwrap();
        if aliases.contains_key(command) {
            let full_raw = aliases.get(command).unwrap().as_str();
            let _raw_args_str;
            (command, _raw_args_str) = full_raw.split_once(" ").unwrap_or((&full_raw, &""));
            // println!("$59${:#?}('{:#?}') (command {:#?})",command,_raw_args_str,parts.split_first().unwrap().0);
            let raw_args_alias2: Vec<&str> = _raw_args_str
                .split_whitespace()
                .into_iter()
                .peekable()
                .collect();
            // for i in

            raw_args_alias = raw_args_alias2.iter().map(|s| s.to_string()).collect();
            // _raw_args

            raw_args_alias.append(&mut raw_args);
        // raw_args = _raw_args
        } else {
            raw_args_alias = raw_args.clone();
        }

        let mut args: Vec<String> = vec![];
        for arg in raw_args_alias {
            // expend each arg
            if arg.contains(&"$".to_string()) || arg.contains(&"~".to_string()) {
                // only expend commands with ~ or $
                let new_arg =
                    shellexpand::full_with_context_no_errors(&arg, get_home_dir, get_env2)
                        .to_string();
                args.push(new_arg);
            } else {
                args.push(arg.to_string());
            }
        }

        match command {
            "cd" => {
                let new_dir = args.get(0).map_or("/", |x| &*x);
                let root = Path::new(new_dir);
                if let Err(e) = env::set_current_dir(&root) {
                    eprintln!("{}", e);
                }

                previous_command = None;
            }
            "source" => {
                for file in args.into_iter() {
                    let filename = file.clone();
                    let file_err = File::open(file);
                    let bufio = match file_err {
                        Err(_) => {
                            println!("cannot open file: {}", filename);
                            continue;
                        }
                        Ok(file) => file,
                    };
                    let reader = BufReader::new(bufio);
                    // reader.lines();
                    for line in reader.lines() {
                        let real_command = line.expect("cannot open file");
                        if real_command.is_empty() {
                            continue;
                        }
                        let ret = handle_command(real_command, aliases, environment); // 1-error status,0-normal status,-1-exit now.
                        if ret == -1 {
                            return ret;
                        }
                    }
                }
            }
            "export" => {
                let the_command = args.get(0);
                // TODO : merge with alias since its the same code
                if the_command.is_none() {
                    println!("environment: {:#?}", environment);
                } else {
                    let exportlist: Vec<&str> = the_command.unwrap().split("=").collect();
                    if exportlist.len() >= 2 {
                        let s1 = exportlist.get(0).unwrap();
                        let s2 = exportlist.get(1).unwrap();
                        environment.insert(s1.to_string(), s2.to_string());
                        env::set_var(s1.to_string(), s2.to_string());
                    } else {
                        if environment.contains_key(&exportlist.get(0).unwrap().to_string()) {
                            println!(
                                "{}",
                                environment
                                    .get(&exportlist.get(0).unwrap().to_string())
                                    .unwrap()
                            )
                        }
                    }
                }
            }
            "alias" => {
                let full_arg = args.join(" ");
                let full_arg_trim = full_arg.trim();
                // full_arg = full_arg.trim();

                // println!("alias {:#?}",args);

                if full_arg_trim.is_empty() {
                    println!("aliases: {:#?}", aliases);
                    continue;
                }
                if full_arg.contains("=") {
                    let (key, value) = full_arg_trim.split_once("=").unwrap();
                    aliases.insert(key.trim().to_string(), value.trim().to_string());
                    // insert key:value to aliases
                } else if aliases.contains_key(full_arg_trim) {
                    println!("{}", aliases.get(full_arg_trim).unwrap())
                }
            }
            "exit" => return -1,
            command => {
                let stdin = previous_command.map_or(Stdio::inherit(), |output: Child| {
                    Stdio::from(output.stdout.unwrap())
                });

                let stdout = if commands.peek().is_some() {
                    // there is another command piped behind this one
                    // prepare to send output to the next command
                    Stdio::piped()
                } else {
                    // there are no more commands piped behind this one
                    // send output to shell stdout
                    Stdio::inherit()
                };
                // println!("running, {:#?} ({})", command, args.join(" ")); // just for debug

                let output = Command::new(command)
                    .args(args)
                    .stdin(stdin)
                    .stdout(stdout)
                    .spawn();

                match output {
                    Ok(output) => {
                        previous_command = Some(output);
                    }

                    Err(e) => {
                        let emessage = e.to_string();
                        let m = match e.kind() {
                            std::io::ErrorKind::NotFound => "command not found",
                            _ => emessage.as_str(),
                        };
                        eprintln!("{}: {}", command, m);
                        previous_command = None;
                        last_code_err = 1;
                    }
                };
            }
        }
    }

    if let Some(mut final_command) = previous_command {
        // block until the final command has finished
        let code = final_command.wait().expect("command error");
        last_code_err = (!code.success()).into();
    }

    return last_code_err;
}
fn get_home_dir() -> Option<String> {
    let home_dir_error = env::var("HOME");
    let home_dir: String;
    match home_dir_error {
        Ok(hd) => home_dir = hd,
        Err(_) => {
            if env::var("userprofile").is_ok() {
                home_dir = env::var("userprofile").unwrap()
            } else {
                home_dir = "/".to_string();
            }
        }
    }
    return Some(home_dir);
}
fn main() {
    // let mut rl = Editor::<()>::new().expect("readline error");

    // fn home_dir_fn() -> Option<String> { Some(home_dir.into()) }
    // let home_dir_fn  = || -> Option<String> {Some((home_dir.into()))};

    let config = Config::builder()
        .history_ignore_space(true)
        .bell_style(rustyline::config::BellStyle::None)
        .completion_type(CompletionType::List)
        .edit_mode(EditMode::Emacs)
        .build();
    let h = lineutils::LineHelper {
        completer: FilenameCompleter::new(),
        highlighter: MatchingBracketHighlighter::new(),
        hinter: HistoryHinter {},
        colored_prompt: "".to_owned(),
        validator: MatchingBracketValidator::new(),
    };
    let mut rl = Editor::with_config(config).expect("error in editor");
    // rl.set_helper(helper);
    rl.set_helper(Some(h));
    rl.bind_sequence(KeyEvent::alt('n'), Cmd::HistorySearchForward);
    rl.bind_sequence(KeyEvent::alt('p'), Cmd::HistorySearchBackward);

    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }
    // let p = format!("{}> ",get_home_dir().unwrap_or("".into()));

    let mut aliases: HashMap<String, String> = HashMap::new();
    let mut environment: HashMap<String, String> = HashMap::new();
    for (key, val) in env::vars_os() {
        if let (Ok(k), Ok(v)) = (key.into_string(), val.into_string()) {
            environment.insert(k, v);
        }
    }
    // let a:HashMap<String,String> = env::vars_os().collect();
    aliases.insert("l".to_string(), "exa".to_string());
    environment.insert("HOME".to_string(), get_home_dir().unwrap_or("".to_string()));
    let mut pwd;
    let homedir = get_home_dir().unwrap();
    // let p = format!("{}> ", pwd.replace(&homedir, "~"));
    // helper.colored_prompt = "".into();
    let ps1 = "${green}{pwd}$reset_color${red_or_green}>$reset_color"; // TODO: actually support PS1
    let green_color = "\x1b[1;32m";
    let reset_color = "\x1b[0m";
    let red_color = "\x1b[0;31m";
    let ps1_resolve = ps1
        .replace("${green}", green_color) // TODO use the shellexpand features
        .replace("$reset_color", reset_color);
    // .replace("{pwd}", pwd.replace(&homedir, "~").as_str());
    // let get_ps1=|success_stat| success_stat;
    let format_prompt = |pwd: &str, red_or_green: &str| {
        ps1_resolve.replace(
            "{pwd}",
            pwd.replace(&homedir, "~").as_str())
                .replace("${red_or_green}", red_or_green)
                // .as_str()
    };
    // rl.helper_mut().unwrap().colored_prompt = format_prompt(&pwd, green_color);
    let mut temp_pwd:String="".to_string();
    loop {
        pwd = env::current_dir()
            .unwrap()
            .into_os_string()
            .into_string()
            .unwrap()
            .replace(&homedir, "~");
        
        if temp_pwd.to_string()!=pwd{
            rl.helper_mut().unwrap().colored_prompt = format_prompt(&pwd, green_color);
        }
        temp_pwd = pwd.clone();
        
        let input = rl.readline(format_prompt(&pwd,"").as_str()); // TODO: use the shellexpand

        match input {
            Ok(line) => {
                let command = line.as_str();
                rl.add_history_entry(command);
                let ret = handle_command(command.to_string(), &mut aliases, &mut environment);
                match ret {
                    -1 => {
                        println!("exiting");
                        break;
                    }
                    0 => {
                        rl.helper_mut().unwrap().colored_prompt = format_prompt(&pwd, green_color);
                    }
                    1 => {
                        rl.helper_mut().unwrap().colored_prompt = format_prompt(&pwd, red_color);
                    }
                    _ => {}
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    rl.save_history("history.txt")
        .expect("error saving to a file");
}
