use regex::Regex;
use rustyline::completion::FilenameCompleter;
use rustyline::hint::HistoryHinter;
use rustyline::validate::MatchingBracketValidator;
use rustyline::{error::ReadlineError, Config};
use rustyline::{Cmd, CompletionType, EditMode, Editor, KeyEvent};
use shlex::Shlex;
mod builtins;
mod cli;
mod lineutils;
mod ps1;
use std::io::{self, Write};
use std::path::Path;
use std::process::exit;
use std::{
    collections::HashMap,
    env,
    fs::File,
    io::{BufRead, BufReader},
    process::{Child, Command, Stdio},
};
fn is_debug() -> bool {
    let arg1 = env::args().nth(1).unwrap_or_default();
    arg1 == "--debug"
}

#[derive(Clone,Debug)]
pub enum CommandStatus {
    EXIT(i32),
    NORMAL,
    Message(String, i32),
    ERROR(i32),
}

fn remove_colors(data: String) -> String {
    let reg = "[\x1B]\\[[()#;?]*(?:[0-9]{1,4}(?:;[0-9]{0,4})*)?[0-9A-ORZcf-nqry=><]"; //[()#;?]*(?:[0-9]{1,4}(?:;[0-9]{0,4})*)?[0-9A-ORZcf-nqry=><]";
    let r: Regex = Regex::new(reg).unwrap();
    // println!("replacing {:#?}",r.replace_all(data.as_str(), ""));
    r.replace_all(data.as_str(), "").to_string() + "     "
}

fn handle_command(
    raw_command: String,
    aliases: &mut HashMap<String, String>,
    environment: &mut HashMap<String, String>,
) -> CommandStatus {
    let env2 = environment.clone();
    let get_env2 = |name: &str| -> Option<&str> { env2.get(name).map(|x| x.as_str()) };

    let builtins_map = builtins::build_map(); // TODO: create only one map, and pass it.
                                              // must be peekable so we know when we are on the last command
    let command = raw_command.split("#").into_iter().next().unwrap();
    // let checker =
    // println!("checker '{}'",checker);
    if command.trim().is_empty() || command.trim().starts_with("#") {
        return CommandStatus::NORMAL;
    }
    let mut shl = Shlex::new(&command);
    let parts: Vec<String> = shl.by_ref().collect();
    // if shl.had_error { None } else { Some(res) }

    // let parts_err = shlex::split(&command);//.expect("shlex split not working");
    if shl.had_error {
        return CommandStatus::Message(format!("syntax error '{}'", command), 1);
    }

    let mut commands_list = parts
        .split(|n| n == "||" || n == "&&" || n == ";")
        .peekable();

    let mut previous_code: CommandStatus = CommandStatus::NORMAL;

    while let Some(logic_parts) = commands_list.next() {
        let mut commands = logic_parts.split(|n| n == "|").peekable();
        // let mut commands = command.trim().split(" | ").peekable();

        let mut previous_command = None;
        // let mut previous_builtin:Option<String> = None; // TODO: pipe from builtins
        while let Some(parts) = commands.next() {
            // let args = parts.split_last().unwrap();
            // if parts.len()>1 // TODO
            let mut raw_args = parts.split_first().unwrap().1.to_vec();
            let mut raw_args_alias: Vec<String>;

            // if (command.is_empty()){continue;}

            // let command = parts.next().unwrap();
            let mut command = parts.first().unwrap().as_str();
            if is_debug() {
                println!("handle command:{:#?} [{:?}]", command, raw_args);
            }
            if aliases.contains_key(command) {
                // TODO alias of alias (like "alias l=exa" then "alias r=l --git" => exa --git )
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

            if builtins_map.contains_key(command) {
                let built = builtins_map.get(command).unwrap();
                let stat = built(&args, environment, aliases);
                previous_code = stat.to_owned();
                match stat {
                    CommandStatus::EXIT(_) => return stat,
                    CommandStatus::ERROR(_) => {}
                    CommandStatus::NORMAL => {}
                    CommandStatus::Message(s, code) => {
                        if code == 0 {
                            println!("{}", s);
                            // previous_builtin = Some(s); // TODO : pipe from builtins
                            // previous_command = Some(s);
                        } else {
                            eprintln!("{}", s)
                        }
                    }
                }
                // previous_command = None;
            } else {
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
                        // last_code_err = CommandStatus::Message(format!("{}: {}", command, m), 1);
                        previous_command = None;
                    }
                };
            }
        }

        if let Some(mut final_command) = previous_command {
            // block until the final command has finished
            let code = final_command.wait().expect("command error");

            io::stdout().flush().unwrap_or_default();
            if code.success() {
                previous_code = CommandStatus::NORMAL;
            } else {
                previous_code = CommandStatus::ERROR(code.code().unwrap_or(1));
            }
            // last_code_err = (!code.success()).into();
        }
    }

    return previous_code;
}

fn source(
    c_files: Vec<String>,
    environment: &mut HashMap<String, String>,
    aliases: &mut HashMap<String, String>,
) -> CommandStatus {
    let mut a = CommandStatus::NORMAL;
    for file in c_files {
        let filename = file.clone();
        let file_err = File::open(file);
        let bufio = match file_err {
            Err(_) => {
                println!("cannot open file: {}", filename);
                a = CommandStatus::ERROR(1);
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
            if let CommandStatus::EXIT(c) = ret {
                return CommandStatus::EXIT(c);
            }
        }
    }
    return a;
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
    let args = cli::parse();

    let binding = std::path::PathBuf::new();
    let c_file = args
        .get_one::<std::path::PathBuf>("rc-file")
        .unwrap_or(&binding);
    let no_rc = args.get_one::<bool>("no-rc").unwrap_or(&false);
    let binding = "".to_string();
    let command_to_run: &String = args.get_one::<String>("command").unwrap_or(&binding);
    let mut aliases: HashMap<String, String> = HashMap::new();
    let mut environment: HashMap<String, String> = HashMap::new();

    for (key, val) in env::vars_os() {
        if let (Ok(k), Ok(v)) = (key.into_string(), val.into_string()) {
            environment.insert(k, v);
        }
    }
    // let a:HashMap<String,String> = env::vars_os().collect();
    aliases.insert("cls".to_string(), "printf '\\033[2J\\033[H'".to_string());
    environment.insert("0".to_string(), "rust-shell".to_string());

    let config = Config::builder()
        .history_ignore_space(true)
        .bell_style(rustyline::config::BellStyle::None)
        .completion_type(CompletionType::List)
        .edit_mode(EditMode::Emacs)
        .build();
    let h = lineutils::LineHelper {
        completer: FilenameCompleter::new(),
        highlighter: lineutils::CommandHighlighter::new(),
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

    let mut pwd;
    let homedir = get_home_dir().unwrap();
    // let p = format!("{}> ", pwd.replace(&homedir, "~"));
    // helper.colored_prompt = "".into();

    let host = hostname::get().unwrap().to_string_lossy().to_string();

    let green_color = "\x1b[1;32m";
    let reset_color = "\x1b[0m";
    let red_color = "\x1b[0;31m";
    environment.insert("green".to_string(), green_color.to_string());
    environment.insert("reset".to_string(), reset_color.to_string());
    environment.insert("red".to_string(), red_color.to_string());

    let default_ps1_prompt = "${green}\\w${reset}${red_or_green}>${reset}";
    environment.insert("PS1".to_string(), default_ps1_prompt.to_string());
    if !command_to_run.is_empty() {
        handle_command(command_to_run.to_string(), &mut aliases, &mut environment);
        return;
    }

    if !no_rc {
        let rcfile = Path::new(&homedir).join(".rustshellrc");
        if rcfile.exists() {
            if is_debug() {
                println!("source rcfile: {:?}", rcfile);
            }
            source(
                vec![rcfile.to_string_lossy().to_string()],
                &mut environment,
                &mut aliases,
            );
        } else {
            if is_debug() {
                println!("could not found rcfile: {:?}", rcfile)
            }
        }
    }
    if c_file.exists() {
        let ret = source(
            vec![c_file.to_string_lossy().to_string()],
            &mut environment,
            &mut aliases,
        );
        if let CommandStatus::EXIT(i) = ret {
            exit(i);
        }
    }
    // .replace("{pwd}", pwd.replace(&homedir, "~").as_str());
    // let get_ps1=|success_stat| success_stat;
    let format_prompt = |pwd: &str,
                         command_status: CommandStatus,
                         environment: &mut HashMap<String, String>,
                         ps1_prompt: &str| {
        let mut exit_status = 0;
        if let CommandStatus::EXIT(n) = command_status {
            exit_status = n;
        } else if let CommandStatus::ERROR(n) = command_status {
            exit_status = n;
        } else if let CommandStatus::Message(_, n) = command_status {
            exit_status = n;
        }
        let red_or_green = if exit_status == 0 {
            green_color
        } else {
            red_color
        };
        environment.insert("exit_status".into(), exit_status.to_string());
        environment.insert("red_or_green".into(), red_or_green.to_string());
        // println!("format prompt: exit:{}",exit_status);

        let ret = ps1::parse(
            shellexpand::env_with_context_no_errors(ps1_prompt, |name: &str| -> Option<&str> {
                environment.get(name).map(|x| x.as_str())
            })
            .to_string(),
            pwd.to_string(),
            host.to_string(), // hostname::get().unwrap().to_string_lossy().to_string(),
        );
        environment.remove("exit_status"); // alow to define custom ps1 with exit_status variable
        environment.remove("red_or_green");
        return ret;
        // .as_str()
    };
    let mut temp_pwd: String = "".to_string();
    let mut temp_ps1: String = "".to_string();

    let mut ps1env;
    let mut ps1_prompt = "";
    let mut ret: CommandStatus;
    let exit_code: i32 = loop {
        pwd = env::current_dir()
            .unwrap()
            .into_os_string()
            .into_string()
            .unwrap()
            .replace(&homedir, "~");

        if temp_pwd.to_string() != pwd {
            rl.helper_mut().unwrap().colored_prompt =
                format_prompt(&pwd, CommandStatus::NORMAL, &mut environment, ps1_prompt);
        }
        if &temp_ps1.to_string()
            != environment
                .get("PS1")
                .unwrap_or(&default_ps1_prompt.to_string().to_string())
        {
            ps1env = environment
                .get("PS1")
                .cloned()
                .unwrap_or(default_ps1_prompt.to_string());
            // println!("{}",ps1env);
            ps1_prompt = ps1env.as_str();
            rl.helper_mut().unwrap().colored_prompt =
                format_prompt(&pwd, CommandStatus::NORMAL, &mut environment, ps1_prompt);
        }
        temp_ps1 = environment
            .get("PS1")
            .unwrap_or(&default_ps1_prompt.to_string())
            .to_string();
        temp_pwd = pwd.clone();
        // let a = &remove_colors(rl.helper_mut().unwrap().colored_prompt.as_mut().to_string());
        let uncolord = &remove_colors(rl.helper_mut().unwrap().colored_prompt.as_mut().to_string());
        // let exit_status:i32 = environment.get("exit_status").unwrap_or(&"".to_string()).parse::<i32>().unwrap_or(0);
        // let uncolord = &remove_colors(format_prompt(&pwd, ret, &mut environment, ps1_prompt));

        let input = rl.readline(&uncolord.trim());
        // let input = rl.readline(format_prompt(&pwd, "", &mut environment,ps1_prompt).as_str()); // TODO: use the shellexpand

        match input {
            Ok(line) => {
                let command = line.as_str();
                rl.add_history_entry(command);
                ret = handle_command(command.to_string(), &mut aliases, &mut environment);
                // io::stdout().flush().unwrap(); // try to flush the stdout for print without new line // TODO add flush to allow print without new line
                let reformat =match ret.clone(){
                    CommandStatus::EXIT(n)=>{println!("exit");break n},
                    CommandStatus::Message(s,i)=>{if i==0{println!("{}",s)} else {eprintln!("{}",s)};1}
                    CommandStatus::ERROR(_)=>{1}
                    CommandStatus::NORMAL=>{1}
                };
                if reformat==1{
                rl.helper_mut().unwrap().colored_prompt =
                    format_prompt(&pwd, ret, &mut environment, ps1_prompt);
            }
        }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break 1;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break 1;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break 1;
            }
        }
    };
    rl.save_history("history.txt")
        .expect("error saving to a file");
    if exit_code != 0 {
        exit(exit_code);
    }
}
