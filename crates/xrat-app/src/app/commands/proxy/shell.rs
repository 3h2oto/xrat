use crate::app::AppError;
use crate::app::commands::output;
use crate::app::context::AppContext;
use crate::cli::{ProxyShellAction, ProxyShellKind, ProxyShellProtocol};

use super::{
    ActiveEndpoints, http_proxy_url, loopback_host, resolve_active_endpoints, socks_proxy_url,
};

pub(super) async fn run(context: &AppContext, action: &ProxyShellAction) -> crate::app::Result<()> {
    match action {
        ProxyShellAction::Enable(args) => {
            let active = resolve_active_endpoints(context).await?;
            let (http_proxy, all_proxy) = proxy_urls(&active, args.protocol)?;
            let kind = detect_shell(args.shell);
            print!("{}", enable_output(kind, &http_proxy, &all_proxy));
            print_status_stderr(&active, Some(http_proxy));
            Ok(())
        }
        ProxyShellAction::Disable(args) => {
            let kind = detect_shell(args.shell);
            print!("{}", disable_output(kind));
            let active = resolve_active_endpoints(context).await?;
            print_status_stderr(&active, None);
            Ok(())
        }
        ProxyShellAction::Status(_) => {
            let active = resolve_active_endpoints(context).await?;
            print_status(&active);
            Ok(())
        }
        ProxyShellAction::Toggle(args) => toggle(context, args.shell).await,
    }
}

pub(super) async fn toggle(
    context: &AppContext,
    shell: Option<ProxyShellKind>,
) -> crate::app::Result<()> {
    let active = resolve_active_endpoints(context).await?;
    let kind = detect_shell(shell);

    if shell_points_at_active(&active) {
        let restored_proxy = restored_proxy_value();
        print!("{}", toggle_off_output(kind));
        print_status_stderr(&active, restored_proxy);
        return Ok(());
    }

    let (http_proxy, all_proxy) = proxy_urls(&active, None)?;
    print!("{}", toggle_on_output(kind, &http_proxy, &all_proxy));
    print_status_stderr(&active, Some(http_proxy));
    Ok(())
}

/// Comment telling the user how to apply the emitted script for their shell.
/// Safe inside an `eval`/`source`, so it can ride on stdout with the script.
fn usage_hint(kind: ProxyShellKind, verb: &str) -> String {
    match kind {
        ProxyShellKind::Bash | ProxyShellKind::Zsh => {
            format!("# apply: eval \"$(xrat proxy shell {verb})\"\n")
        }
        ProxyShellKind::Fish => format!("# apply: xrat proxy shell {verb} | source\n"),
    }
}

fn enable_output(kind: ProxyShellKind, http_proxy: &str, all_proxy: &str) -> String {
    let mut out = usage_hint(kind, "enable");
    out.push_str(&enable_script(kind, http_proxy, all_proxy));
    out
}

fn disable_output(kind: ProxyShellKind) -> String {
    let mut out = usage_hint(kind, "disable");
    out.push_str(&disable_script(kind));
    out
}

fn toggle_off_output(kind: ProxyShellKind) -> String {
    let mut out = usage_hint(kind, "toggle");
    out.push_str(&restore_script(kind));
    out
}

fn toggle_on_output(kind: ProxyShellKind, http_proxy: &str, all_proxy: &str) -> String {
    let mut out = usage_hint(kind, "toggle");
    out.push_str(&capture_script(kind));
    out.push_str(&enable_script(kind, http_proxy, all_proxy));
    out
}

/// Resolve the `http_proxy`/`https_proxy` and `all_proxy` URLs from active
/// endpoints. By default HTTP is preferred for `http_proxy`/`https_proxy`
/// (SOCKS fallback); SOCKS is preferred for `all_proxy` (HTTP fallback). An
/// explicit protocol requires the matching inbound for both variables.
fn proxy_urls(
    active: &ActiveEndpoints,
    protocol: Option<ProxyShellProtocol>,
) -> crate::app::Result<(String, String)> {
    match protocol {
        Some(protocol) => explicit_proxy_url(active, protocol)
            .map(|url| (url.clone(), url))
            .ok_or_else(|| unavailable_protocol_error(protocol)),
        None => {
            let http_url = active
                .http
                .as_ref()
                .map(|(host, port)| http_proxy_url(host, *port))
                .or_else(|| {
                    active
                        .socks
                        .as_ref()
                        .map(|(host, port)| socks_proxy_url(host, *port))
                });
            let all_url = active
                .socks
                .as_ref()
                .map(|(host, port)| socks_proxy_url(host, *port))
                .or_else(|| {
                    active
                        .http
                        .as_ref()
                        .map(|(host, port)| http_proxy_url(host, *port))
                });

            match (http_url, all_url) {
                (Some(http_url), Some(all_url)) => Ok((http_url, all_url)),
                _ => Err(AppError::InvalidArgument(
                    "no active HTTP or SOCKS inbound; start a runtime with `xrat connect <id>`"
                        .to_string(),
                )),
            }
        }
    }
}

fn explicit_proxy_url(active: &ActiveEndpoints, protocol: ProxyShellProtocol) -> Option<String> {
    match protocol {
        ProxyShellProtocol::Http => active
            .http
            .as_ref()
            .map(|(host, port)| http_proxy_url(host, *port)),
        ProxyShellProtocol::Socks5 => active
            .socks
            .as_ref()
            .map(|(host, port)| format!("socks5://{}:{port}", loopback_host(host))),
        ProxyShellProtocol::Socks5h => active
            .socks
            .as_ref()
            .map(|(host, port)| format!("socks5h://{}:{port}", loopback_host(host))),
    }
}

fn unavailable_protocol_error(protocol: ProxyShellProtocol) -> AppError {
    let (name, setting) = match protocol {
        ProxyShellProtocol::Http => ("HTTP", "[runtime.http].enabled"),
        ProxyShellProtocol::Socks5 | ProxyShellProtocol::Socks5h => {
            ("SOCKS", "[runtime.socks].enabled")
        }
    };
    AppError::InvalidArgument(format!(
        "{name} inbound is not active; set {setting} = true and reconnect, or omit the protocol to use an available inbound"
    ))
}

const VARS: [&str; 3] = ["http_proxy", "https_proxy", "all_proxy"];
const ALL_VARS: [&str; 6] = [
    "http_proxy",
    "HTTP_PROXY",
    "https_proxy",
    "HTTPS_PROXY",
    "all_proxy",
    "ALL_PROXY",
];

fn enable_script(kind: ProxyShellKind, http_proxy: &str, all_proxy: &str) -> String {
    let value_for = |name: &str| -> &str {
        if name == "all_proxy" {
            all_proxy
        } else {
            http_proxy
        }
    };

    let mut out = String::new();
    for name in VARS {
        let value = value_for(name);
        let upper = name.to_uppercase();
        match kind {
            ProxyShellKind::Bash | ProxyShellKind::Zsh => {
                out.push_str(&format!("export {name}=\"{value}\"\n"));
                out.push_str(&format!("export {upper}=\"{value}\"\n"));
            }
            ProxyShellKind::Fish => {
                out.push_str(&format!("set -gx {name} \"{value}\"\n"));
                out.push_str(&format!("set -gx {upper} \"{value}\"\n"));
            }
        }
    }
    out
}

fn disable_script(kind: ProxyShellKind) -> String {
    let mut out = String::new();
    for name in VARS {
        let upper = name.to_uppercase();
        match kind {
            ProxyShellKind::Bash | ProxyShellKind::Zsh => {
                out.push_str(&format!("unset {name}\n"));
                out.push_str(&format!("unset {upper}\n"));
            }
            ProxyShellKind::Fish => {
                out.push_str(&format!("set -e {name}\n"));
                out.push_str(&format!("set -e {upper}\n"));
            }
        }
    }
    out
}

fn capture_script(kind: ProxyShellKind) -> String {
    let mut out = String::new();
    for name in ALL_VARS {
        let old_name = old_var_name(name);
        let had_name = had_var_name(name);
        match kind {
            ProxyShellKind::Bash | ProxyShellKind::Zsh => {
                out.push_str(&format!("if [ \"${{{name}+x}}\" ]; then\n"));
                out.push_str(&format!("  export {old_name}=\"${name}\"\n"));
                out.push_str(&format!("  export {had_name}=1\n"));
                out.push_str("else\n");
                out.push_str(&format!("  unset {old_name}\n"));
                out.push_str(&format!("  export {had_name}=0\n"));
                out.push_str("fi\n");
            }
            ProxyShellKind::Fish => {
                out.push_str(&format!("if set -q {name}\n"));
                out.push_str(&format!("    set -gx {old_name} \"${name}\"\n"));
                out.push_str(&format!("    set -gx {had_name} 1\n"));
                out.push_str("else\n");
                out.push_str(&format!("    set -e {old_name}\n"));
                out.push_str(&format!("    set -gx {had_name} 0\n"));
                out.push_str("end\n");
            }
        }
    }
    out
}

fn restore_script(kind: ProxyShellKind) -> String {
    let mut out = String::new();
    for name in ALL_VARS {
        let old_name = old_var_name(name);
        let had_name = had_var_name(name);
        match kind {
            ProxyShellKind::Bash | ProxyShellKind::Zsh => {
                out.push_str(&format!("if [ \"${{{had_name}:-0}}\" = \"1\" ]; then\n"));
                out.push_str(&format!("  export {name}=\"${old_name}\"\n"));
                out.push_str("else\n");
                out.push_str(&format!("  unset {name}\n"));
                out.push_str("fi\n");
                out.push_str(&format!("unset {old_name}\n"));
                out.push_str(&format!("unset {had_name}\n"));
            }
            ProxyShellKind::Fish => {
                out.push_str(&format!("if test \"${had_name}\" = 1\n"));
                out.push_str(&format!("    set -gx {name} \"${old_name}\"\n"));
                out.push_str("else\n");
                out.push_str(&format!("    set -e {name}\n"));
                out.push_str("end\n");
                out.push_str(&format!("set -e {old_name}\n"));
                out.push_str(&format!("set -e {had_name}\n"));
            }
        }
    }
    out
}

fn old_var_name(name: &str) -> String {
    format!("XRAT_PROXY_OLD_{name}")
}

fn had_var_name(name: &str) -> String {
    format!("XRAT_PROXY_HAD_{name}")
}

/// Detect the target shell: explicit override, otherwise shared platform
/// detection ($SHELL, then parent process, defaulting to bash).
fn detect_shell(override_kind: Option<ProxyShellKind>) -> ProxyShellKind {
    if let Some(kind) = override_kind {
        return kind;
    }
    match xrat_support::platform::detect_shell() {
        xrat_support::platform::Shell::Bash => ProxyShellKind::Bash,
        xrat_support::platform::Shell::Zsh => ProxyShellKind::Zsh,
        xrat_support::platform::Shell::Fish => ProxyShellKind::Fish,
    }
}

fn print_status(active: &ActiveEndpoints) {
    println!("{}", status_text(active));
}

/// Status output for the auto-status printed after enable/disable/toggle. Goes
/// to stderr so the eval-able script on stdout stays clean. The expected proxy
/// value is supplied because this process cannot observe changes made by the
/// emitted script in the caller's shell.
fn print_status_stderr(active: &ActiveEndpoints, expected_proxy: Option<String>) {
    eprintln!("{}", status_text_for(active, expected_proxy));
}

fn status_text(active: &ActiveEndpoints) -> String {
    status_text_for(active, current_proxy_value())
}

fn status_text_for(active: &ActiveEndpoints, current: Option<String>) -> String {
    let color = output::color_enabled();

    let active_hosts = active_hostports(active);
    let pointing = current
        .as_deref()
        .map(|value| points_at_active(value, &active_hosts))
        .unwrap_or(false);

    let state = match (&current, pointing) {
        (Some(_), true) => "shell points at active xrat endpoints",
        (Some(_), false) => "shell proxy is set but does not match active xrat endpoints",
        (None, _) => "shell has no proxy environment set",
    };

    output::format_kv(
        Some("Proxy shell"),
        &[
            ("status", state.to_string()),
            (
                "http_proxy",
                current.clone().unwrap_or_else(|| "-".to_string()),
            ),
            (
                "active",
                if active_hosts.is_empty() {
                    "-".to_string()
                } else {
                    active_hosts.join(", ")
                },
            ),
        ],
        color,
    )
}

fn current_proxy_value() -> Option<String> {
    current_proxy_value_with_env(&xrat_support::env::SystemEnvVars)
}

fn current_proxy_value_with_env(env: &dyn xrat_support::env::EnvVars) -> Option<String> {
    ["http_proxy", "HTTP_PROXY", "all_proxy", "ALL_PROXY"]
        .into_iter()
        .find_map(|name| env.get((name).as_ref()).ok())
}

fn restored_proxy_value() -> Option<String> {
    restored_proxy_value_with_env(&xrat_support::env::SystemEnvVars)
}

fn restored_proxy_value_with_env(env: &dyn xrat_support::env::EnvVars) -> Option<String> {
    ["http_proxy", "HTTP_PROXY", "all_proxy", "ALL_PROXY"]
        .into_iter()
        .find_map(|name| {
            (env.get((had_var_name(name)).as_ref()).ok().as_deref() == Some("1"))
                .then(|| env.get((old_var_name(name)).as_ref()).ok())
                .flatten()
        })
}

fn shell_points_at_active(active: &ActiveEndpoints) -> bool {
    shell_points_at_active_with_env(active, &xrat_support::env::SystemEnvVars)
}

fn shell_points_at_active_with_env(
    active: &ActiveEndpoints,
    env: &dyn xrat_support::env::EnvVars,
) -> bool {
    let active_hosts = active_hostports(active);
    if active_hosts.is_empty() {
        return false;
    }

    ALL_VARS.iter().any(|name| {
        env.get((name).as_ref())
            .map(|value| points_at_active(&value, &active_hosts))
            .unwrap_or(false)
    })
}

fn active_hostports(active: &ActiveEndpoints) -> Vec<String> {
    [active.http.as_ref(), active.socks.as_ref()]
        .into_iter()
        .flatten()
        .map(|(host, port)| format!("{}:{port}", loopback_host(host)))
        .collect()
}

fn points_at_active(value: &str, active_hosts: &[String]) -> bool {
    active_hosts.iter().any(|hostport| value.contains(hostport))
}

#[cfg(test)]
mod tests;
