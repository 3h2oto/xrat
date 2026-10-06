use clap::Parser;

use crate::cli::{Cli, Command, TunAction};

#[test]
fn parses_tun_subcommands() {
    for (subcommand, enabled) in [("enable", true), ("disable", false)] {
        let cli = Cli::parse_from(["xrat", "tun", subcommand]);
        match cli.command {
            Command::Tun(args) => assert!(if enabled {
                matches!(args.action, TunAction::Enable(_))
            } else {
                matches!(args.action, TunAction::Disable(_))
            }),
            _ => panic!("expected tun command"),
        }
    }
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

#[test]
fn parses_tun_mode_json() {
    for command in ["enable", "disable"] {
        let cli = Cli::try_parse_from(["xrat", "tun", command, "--json"]).unwrap();
        let Command::Tun(args) = cli.command else {
            panic!("expected tun");
        };
        match args.action {
            TunAction::Enable(mode) | TunAction::Disable(mode) => assert!(mode.json),
            _ => panic!("expected a mode command"),
        }
    }
}
