use clap::{Command, crate_version, arg};

pub fn parse() -> clap::ArgMatches{
    let matches = Command::new("rust-shell")
        .about("unix shell written in rust")
        .version(crate_version!())
        .subcommand_required(false)
        .arg_required_else_help(false)
        .author("matan-h")
        .arg(
         arg!(-'f' --"rc-file" <PATH>).required(false).value_parser(clap::value_parser!(std::path::PathBuf)).help("rc file path")
      )
      .arg(arg!(--"no-rc").required(false).value_parser(clap::builder::ValueParser::bool()).help("do not run the rc file"))
      .arg(arg!(-'#' --"debug").required(false))
      
      ;
      return matches.get_matches();
        // Query subcommand
        //
        // Only a few of its arguments are implemented below.
        
}