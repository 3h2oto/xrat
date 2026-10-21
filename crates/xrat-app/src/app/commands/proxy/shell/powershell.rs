const VARIABLES: [&str; 3] = ["HTTP_PROXY", "HTTPS_PROXY", "ALL_PROXY"];

fn literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

pub(super) fn capture() -> String {
    let mut script = String::new();
    for name in VARIABLES {
        script.push_str(&format!(
            "if (-not (Test-Path Env:XRAT_PROXY_HAD_{name})) {{\n  if (Test-Path Env:{name}) {{\n    $env:XRAT_PROXY_OLD_{name} = $env:{name}\n    $env:XRAT_PROXY_HAD_{name} = '1'\n  }} else {{\n    $env:XRAT_PROXY_HAD_{name} = '0'\n  }}\n}}\n"
        ));
    }
    script
}

pub(super) fn enable(http_proxy: &str, all_proxy: &str) -> String {
    let mut script = capture();
    for (name, value) in [
        ("HTTP_PROXY", http_proxy),
        ("HTTPS_PROXY", http_proxy),
        ("ALL_PROXY", all_proxy),
    ] {
        script.push_str(&format!("$env:{name} = {}\n", literal(value)));
    }
    script
}

pub(super) fn restore() -> String {
    let mut script = String::new();
    for name in VARIABLES {
        script.push_str(&format!(
            "if (Test-Path Env:XRAT_PROXY_HAD_{name}) {{\n  if ($env:XRAT_PROXY_HAD_{name} -eq '1') {{\n    $env:{name} = $env:XRAT_PROXY_OLD_{name}\n  }} else {{\n    Remove-Item Env:{name} -ErrorAction SilentlyContinue\n  }}\n  Remove-Item Env:XRAT_PROXY_OLD_{name} -ErrorAction SilentlyContinue\n  Remove-Item Env:XRAT_PROXY_HAD_{name} -ErrorAction SilentlyContinue\n}}\n"
        ));
    }
    script
}

#[cfg(test)]
#[path = "powershell_tests.rs"]
mod tests;
