use clap::Parser;

use crate::cli::{Cli, Command};

#[test]
fn parses_upgrade_tun_and_daemon_flags() {
    let cli = Cli::try_parse_from(["xrat", "upgrade", "--tun-setup"]).unwrap();
    let Command::Upgrade(args) = cli.command else {
        panic!("expected upgrade command");
    };
    assert!(args.tun_setup);
    assert!(!args.no_tun_setup);
    assert!(!args.no_restart_daemon);

    let cli =
        Cli::try_parse_from(["xrat", "upgrade", "--no-tun-setup", "--no-restart-daemon"]).unwrap();
    let Command::Upgrade(args) = cli.command else {
        panic!("expected upgrade command");
    };
    assert!(!args.tun_setup);
    assert!(args.no_tun_setup);
    assert!(args.no_restart_daemon);

    // Conflict between --tun-setup and --no-tun-setup
    assert!(Cli::try_parse_from(["xrat", "upgrade", "--tun-setup", "--no-tun-setup"]).is_err());
}
