use clap::{Command, crate_version, arg};

pub fn parse() -> clap::ArgMatches{
    let matches = Command::new("rust-shell")
        .about("unix shell written in rust")
        .version(crate_version!())
        .subcommand_required(false)
        .arg_required_else_help(false)
        .author("matan-h")
        .arg(
         arg!(-'c' --"rc-file" <PATH>).required(false).value_parser(clap::value_parser!(std::path::PathBuf))
      )
      .arg(arg!(-'#' --"debug").required(false))
      
      ;
      return matches.get_matches();
        // Query subcommand
        //
        // Only a few of its arguments are implemented below.
        
}