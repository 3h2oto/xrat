use super::prelude::*;
use crate::app::commands::output;

pub fn run(args: &ValidateArgs) -> crate::app::Result<()> {
    let diagnostics = collect_errors(args);
    let color = output::color_enabled();

    match args.format {
        ValidateFormat::Human => {
            if diagnostics.is_empty() {
                println!(
                    "{}",
                    output::success(format!("{} is valid.", args.path.display()), color)
                );
            } else {
                eprintln!("{}", render_human(args, &diagnostics, color));
            }
        }
        ValidateFormat::Json => {
            let report = serde_json::json!({
                "path": args.path.display().to_string(),
                "valid": diagnostics.is_empty(),
                "errors": diagnostics,
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".to_string())
            );
        }
    }

    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(AppError::InvalidArgument(format!(
            "{} is invalid",
            args.path.display()
        )))
    }
}

pub(crate) fn render_human(args: &ValidateArgs, diagnostics: &[Diagnostic], color: bool) -> String {
    let title = format!(
        "{} has {} validation error(s):",
        args.path.display(),
        diagnostics.len()
    );

    let mut lines = Vec::with_capacity(diagnostics.len() * 4 + 1);
    lines.push(output::style_text(&title, Style::Dim, color));

    for diagnostic in diagnostics {
        lines.push(String::new());
        let heading = if diagnostic.field.is_empty() {
            diagnostic.problem.clone()
        } else {
            format!(
                "{} {}",
                output::style_text(&diagnostic.field, Style::Red, color),
                diagnostic.problem
            )
        };
        lines.push(format!("  {heading}"));
        lines.push(format!(
            "    {} {}",
            output::style_text("why:", Style::Dim, color),
            diagnostic.reason
        ));
        lines.push(format!(
            "    {} {}",
            output::style_text("fix:", Style::Dim, color),
            diagnostic.fix
        ));
    }

    lines.join("\n")
}

/// Collect all validation findings. Structural problems (missing/non-file path,
/// parse failure) short-circuit deeper checks since there is no config to lint.
pub(crate) fn collect_errors(args: &ValidateArgs) -> Vec<Diagnostic> {
    let mut errors = Vec::new();

    if !args.path.exists() {
        errors.push(Diagnostic::structural(
            format!("config file does not exist: {}", args.path.display()),
            "validation needs an existing config file to read.",
            "create the file or pass the correct path; run `xrat init` to generate a default config.",
        ));
        return errors;
    }
    if !args.path.is_file() {
        errors.push(Diagnostic::structural(
            format!("config path is not a file: {}", args.path.display()),
            "the path points at a directory or special file, not a readable config.",
            "pass the path to the config.toml file itself.",
        ));
        return errors;
    }

    let contents = match std::fs::read_to_string(&args.path) {
        Ok(contents) => contents,
        Err(err) => {
            errors.push(Diagnostic::structural(
                format!("config file could not be read: {err}"),
                "the file exists but could not be opened for reading.",
                "check file permissions and that the path points at a regular file.",
            ));
            return errors;
        }
    };

    match toml::from_str::<toml::Value>(&contents) {
        Ok(value) => check_known_fields(&value, &mut errors),
        Err(err) => {
            errors.push(Diagnostic::structural(
                format!("config file is not valid TOML: {err}"),
                "the file could not be parsed as TOML syntax.",
                "fix the reported syntax error, including quoting, commas, and table headers.",
            ));
            return errors;
        }
    }

    if !errors.is_empty() {
        return errors;
    }

    match crate::app::config::load(&args.path) {
        Ok(config) => validate_config(&config, &mut errors),
        Err(err) => errors.push(Diagnostic::structural(
            format!("config failed to parse: {err}"),
            "the file is not valid TOML or has a field with the wrong type or an unknown enum value.",
            "fix the reported syntax or value; check quoting, table headers, and that enum fields use accepted values.",
        )),
    }

    errors
}
