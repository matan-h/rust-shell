use chrono::{DateTime, Local};
use std::env;
use std::ops::Range;
use std::path::PathBuf;
use std::process::Command;

use crate::is_debug;


// PS1 parser. most of it written by chatgpt. (took me more then 3h to debug it and took me 10m to ask the chatgpt and encourage it to complete the program , so now most of it written by me :| )
pub fn run_command(full_command: String) -> String {
    let (command, args) = full_command
        .split_once(" ")
        .unwrap_or((full_command.as_str(), ""));
    let output_err = Command::new(command).args(args.split_whitespace()).output();
    //  .expect("failed to execute process in PS1: ");
    // silent fail
    return match output_err {
        Ok(output) => String::from_utf8_lossy(&output.stdout).to_string(),
        Err(_) => "".to_string(),
    };
}
pub fn parse(ps1: String, pwd: String, hostname: String) -> String {
    let cwd = PathBuf::from(pwd);
    let mut new_ps1 = ps1.clone();
    let now: DateTime<Local> = Local::now();
    let mut delay = 0;
    let mut ascii_undelay = 0;
    // let mut commands: HashMap<String, Range<i32>> = HashMap::new();

    let mut is_dollar = false;
    let mut is_command = false;
    let mut command_tmp = "".to_string();
    let mut command_start = 0;
    let mut command_end;
    for (i, c) in ps1.chars().enumerate() {
        if c != '\\' {
            if c == ')' && is_command {
                command_end = i;
                if is_debug() {
                    println!(
                        "collected command '{}' (range:{:#?})",
                        command_tmp,
                        Range {
                            start: command_start,
                            end: command_end
                        }
                    );
                }

                let the_command = new_ps1
                    .get(Range {
                        start: command_start + delay - ascii_undelay + 1,
                        end: command_end + delay - ascii_undelay,
                    })
                    .unwrap_or("");
                let output = run_command(the_command.to_string());
                if output.is_empty() {
                    continue;
                }
                let replace_range = Range {
                    start: command_start + delay - ascii_undelay - 1,
                    end: command_end + delay - ascii_undelay + 1,
                };
                if is_debug() {
                    println!("delay:{}", delay)
                };
                delay += output
                    .clone()
                    .len()
                    .abs_diff(replace_range.end - replace_range.start)
                    - 2;
                if is_debug() {
                    println!("new-delay:{}", delay)
                };
                new_ps1.replace_range(replace_range, &output.trim());

                // delay+=the_command.len().abs_diff(output.len());
                command_tmp = "".to_string();
                is_dollar = false;
                is_command = false;
            }
            if is_command {
                command_tmp.push(c);
            }
            if c == '(' && is_dollar {
                is_command = true;
                command_start = i
            }
            if c == '$' {
                is_dollar = true;
            }
            continue;
        }
        let next_char = ps1.chars().nth(i + 1).unwrap_or('\\');
        let replacement: String = match next_char {
            'a' => "\x07".to_string(),
            'd' => now.format("%a %b %d").to_string(),
            'e' => "\x1B".to_string(),
            'h' => {
                let hostname = hostname.split('.').next().unwrap_or(&hostname);
                hostname.to_string()
            }
            'H' => hostname.to_string(),
            'j' => env::var("JOBS").unwrap_or_else(|_| "0".to_string()),
            'l' => {
                let tty =
                    PathBuf::from(&env::var("TTY").unwrap_or_else(|_| "/dev/tty".to_string()));
                let tty_basename = tty.file_name().unwrap().to_string_lossy();
                tty_basename.to_string()
            }
            'n' => "\n".to_string(),
            'r' => "\r".to_string(),
            's' => {
                let shell =
                    PathBuf::from(env::var("0").unwrap_or_else(|_| "rust-shell".to_string()));
                let shell_basename = shell.file_name().unwrap().to_string_lossy();
                shell_basename.to_string()
            }
            't' => now.format("%T").to_string(),
            'T' => now.format("%r").to_string(),
            '@' => now.format("%I:%M %p").to_string(),
            'A' => now.format("%R").to_string(),
            'u' => env::var("USER").unwrap_or_else(|_| "user".to_string()),
            // 'v' =>  env::var("BASH_VERSION").unwrap_or_else(|_| "5.0.16(1)-release".to_string()),
            // 'V' => env::var("BASH_RELEASE").unwrap_or_else(|_| "5.0".to_string()),
            'w' => {
                let home_path = env::var("HOME").unwrap_or("/home/user".to_string());
                let home_path = PathBuf::from(&home_path);
                let cwd_display = cwd.strip_prefix(&home_path).unwrap_or(&cwd);
                cwd_display.to_string_lossy().to_string()
            }
            'W' => {
                let cwd_basename = cwd.file_name().unwrap().to_string_lossy();
                let home_path = env::var("HOME").unwrap_or("/home/user".to_string());
                let home_path = PathBuf::from(&home_path);
                let home_basename = home_path.file_name().unwrap().to_string_lossy();
                let cwd_basename = if cwd_basename == home_basename {
                    "~".to_string()
                } else {
                    cwd_basename.to_string()
                };
                cwd_basename
            }
            '!' => env::var("HISTCMD").unwrap_or("0".to_string()),
            '#' => env::var("LINENO").unwrap_or("0".to_string()),
            '$' => {
                let effective_uid = env::var("EUID").unwrap_or("1000".to_string());
                if effective_uid == "0" {
                    "#".to_string()
                } else {
                    "$".to_string()
                }
            }
            '\\' => "\\".to_string(),
            '[' => "".to_string(),
            ']' => "".to_string(),
            _ => continue,
        };
        // println!("new-ps1: '{}' from-start:'{:#?}' to-end:'{:#?}'",new_ps1,new_ps1.get(..i+delay+ascii_undelay),new_ps1.get(i+delay+ascii_undelay..));
        // println!("replace {} -> {}",new_ps1.get(i+delay-ascii_undelay..i+delay+2-ascii_undelay).unwrap(),replacement.escape_debug());
        // Replace the escape sequence with the replacement string
        new_ps1.replace_range(
            i + delay - ascii_undelay..i + delay + 2 - ascii_undelay,
            &replacement,
        );
        if replacement == "\x1B" || replacement == "\r" {
            ascii_undelay += 1;
        } else {
            if replacement.len() >= 2 {
                delay += replacement.len() - 2;
            } else if replacement.len() == 1 {
                ascii_undelay += 1;
            } else {
                ascii_undelay += 2;
            }
        }
    }
    if new_ps1.contains("%{\x1B") && new_ps1.contains("%}"){
        new_ps1 = new_ps1.replace("%{", "").replace("%}", ""); // use %{color%} in the ps1
    }

    return new_ps1;
}
