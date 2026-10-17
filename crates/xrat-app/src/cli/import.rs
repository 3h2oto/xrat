use clap::Args;
use xrat_support::http::{HeaderName, HeaderValue};

#[derive(Debug, Args)]
#[command(about = "Import a subscription URL, file, or raw text into SQLite.")]
pub struct ImportArgs {
    #[arg(help = "Subscription source: a URL, local file path, or raw subscription text.")]
    pub input: String,

    #[arg(
        long,
        conflicts_with = "headers",
        help = "Read a local file of subscription URLs and fetch each once."
    )]
    pub url_list: bool,

    #[arg(
        short = 'n',
        long,
        value_name = "NAME",
        value_parser = parse_name,
        help = "Name for the imported subscription source."
    )]
    pub name: Option<String>,

    #[arg(
        long = "header",
        value_name = "NAME:VALUE",
        value_parser = parse_header,
        help = "HTTP header for URL subscriptions (repeatable). Saved for future updates; replaces stored headers."
    )]
    pub headers: Vec<(HeaderName, HeaderValue)>,
}

fn parse_header(value: &str) -> Result<(HeaderName, HeaderValue), String> {
    let (name, value) = value
        .split_once(':')
        .ok_or("header must have the form NAME:VALUE")?;
    let name = HeaderName::from_bytes(name.as_bytes()).map_err(|_| "invalid HTTP header name")?;
    let value = value.trim_matches([' ', '\t']);
    let mut value = HeaderValue::from_str(value).map_err(|_| "invalid HTTP header value")?;
    value.set_sensitive(true);
    Ok((name, value))
}

fn parse_name(value: &str) -> Result<String, String> {
    let name = value.trim();
    if name.is_empty() {
        return Err("name must not be empty".to_string());
    }
    Ok(name.to_string())
}
