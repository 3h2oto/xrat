use super::*;

pub fn parse() -> Cli {
    Cli::try_parse().unwrap_or_else(|e| {
        use clap::error::ErrorKind;
        if matches!(e.kind(), ErrorKind::MissingRequiredArgument) && std::env::args_os().len() == 1
        {
            let mut args: Vec<String> = std::env::args().collect();
            args.push("tui".to_string());
            Cli::parse_from(args)
        } else {
            e.exit()
        }
    })
}
