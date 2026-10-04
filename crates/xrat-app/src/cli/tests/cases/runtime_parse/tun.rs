use clap::Parser;

use crate::cli::{Cli, Command, TunAction};

#[test]
fn parses_tun_subcommands() {
    let status = Cli::parse_from(["xrat", "tun", "status"]);
    match status.command {
        Command::Tun(args) => assert!(matches!(args.action, TunAction::Status(_))),
        _ => panic!("expected tun command"),
    }

    let status_json = Cli::parse_from(["xrat", "tun", "status", "--json"]);
    match status_json.command {
        Command::Tun(args) => match args.action {
            TunAction::Status(status) => assert!(status.json),
            _ => panic!("expected tun status action"),
        },
        _ => panic!("expected tun command"),
    }

    let setup = Cli::parse_from(["xrat", "tun", "setup", "--dry-run"]);
    match setup.command {
        Command::Tun(args) => match args.action {
            TunAction::Setup(setup) => assert!(setup.dry_run),
            _ => panic!("expected tun setup action"),
        },
        _ => panic!("expected tun command"),
    }
}
