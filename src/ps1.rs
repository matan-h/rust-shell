use std::env;
use std::path::PathBuf;
use chrono::{DateTime, Local};
// PS1 parser. most of it written by chatgpt. (took me 2h to debug it and took me 10m to ask the chatgpt and encourage it to complete the program )
pub fn parse(ps1:String,pwd:String,hostname:String)->String {
    let cwd = PathBuf::from(pwd);
let mut new_ps1 = ps1.clone();
let now: DateTime<Local> = Local::now();
let mut delay = 0;
let mut ascii_undelay = 0;
for (i, c) in ps1.chars().enumerate() {
    if c != '\\' {
        continue;
    }
    let next_char = ps1.chars().nth(i + 1).unwrap_or('\\');
    let replacement:String = match next_char {
        'a' => "\x07".to_string(),
        'd' => now.format("%a %b %d").to_string(),
        'e' => "\x1B".to_string(),
        'h' => {
            let hostname = hostname.split('.').next().unwrap_or(&hostname);
            hostname.to_string()
        },
        'H' => {
            hostname.to_string()
        },
        'j' => env::var("JOBS").unwrap_or_else(|_| "0".to_string()),
        'l' => {
            let tty = PathBuf::from(&env::var("TTY").unwrap_or_else(|_| "/dev/tty".to_string()));
            let tty_basename = tty.file_name().unwrap().to_string_lossy();
            tty_basename.to_string()
        },
        'n' => "\n".to_string(),
        'r' => "\r".to_string(),
        's' => {
            let shell = PathBuf::from(env::var("0").unwrap_or_else(|_| "rust-shell".to_string()));
            let shell_basename = shell.file_name().unwrap().to_string_lossy();
            shell_basename.to_string()
        },
        't' => now.format("%T").to_string(),
        'T' => now.format("%r").to_string(),
        '@' =>  now.format("%I:%M %p").to_string(),
        'A' =>  now.format("%R").to_string(),
        'u' =>  env::var("USER").unwrap_or_else(|_| "user".to_string()),
        // 'v' =>  env::var("BASH_VERSION").unwrap_or_else(|_| "5.0.16(1)-release".to_string()),
        // 'V' => env::var("BASH_RELEASE").unwrap_or_else(|_| "5.0".to_string()),
        'w' => {
            let home_path = env::var("HOME").unwrap_or( "/home/user".to_string());
            let home_path = PathBuf::from(&home_path);
            let cwd_display =cwd.strip_prefix(&home_path).unwrap_or(&cwd);
            cwd_display.to_string_lossy().to_string()
            },
        'W' => {

            let cwd_basename =cwd.file_name().unwrap().to_string_lossy();
            let home_path = env::var("HOME").unwrap_or( "/home/user".to_string());
            let home_path = PathBuf::from(&home_path);
            let home_basename = home_path.file_name().unwrap().to_string_lossy();
            let cwd_basename = if cwd_basename == home_basename { "~".to_string() } else { cwd_basename.to_string() };
            cwd_basename
            },
            '!' => env::var("HISTCMD").unwrap_or( "0".to_string()),
            '#' => env::var("LINENO").unwrap_or( "0".to_string()),
            '$' => {
            let effective_uid = env::var("EUID").unwrap_or( "1000".to_string());
            if effective_uid == "0" {
            "#".to_string()
            } else {
            "$".to_string()
            }
            },
            '\\' => "\\".to_string(),
            '['=>"".to_string(),
            ']'=>"".to_string(),
            _ => continue,
            };
            // println!("replace {} -> {}",new_ps1.get(i+delay-ascii_undelay..i+delay+2-ascii_undelay).unwrap(),replacement.escape_debug());
            // Replace the escape sequence with the replacement string
            new_ps1.replace_range(i+delay-ascii_undelay..i+delay+2-ascii_undelay, &replacement);
            if replacement=="\x1B"||replacement=="\r"{
                ascii_undelay+=1;
                
            }
            else{
                if replacement.len()>=2{
                    delay += replacement.len()-2;
                }
                else if replacement.len()==1{                    
                    ascii_undelay+=1;
                }
                else{
                    ascii_undelay+=2;
                }
            }
        }
        
        
        return new_ps1;
        
    }
