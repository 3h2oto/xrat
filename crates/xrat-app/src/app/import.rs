use crate::app::AppError;
use crate::app::input::source::{read_input, read_input_async};
use xrat_config::parse_text;
use xrat_db::ImportSource;
use xrat_db::SourceKind;
use xrat_db::{Database, ImportSummary};
use xrat_model::Node;
use xrat_support::decode::decode_or_raw_text;
use xrat_support::url::looks_like_url;

mod subscription;
pub use subscription::import_subscription;

pub fn load_nodes(input: &str) -> crate::app::Result<(ImportSource, Vec<Node>)> {
    let (source, input_data) = read_input(input)?;
    Ok((source, parse_nodes(&input_data)?))
}

pub async fn load_nodes_async(input: &str) -> crate::app::Result<(ImportSource, Vec<Node>)> {
    let (source, input_data) = read_input_async(input).await?;
    Ok((source, parse_nodes(&input_data)?))
}

fn parse_nodes(input_data: &[u8]) -> crate::app::Result<Vec<Node>> {
    let config_text = decode_or_raw_text(input_data)?;
    reject_raw_json_config(&config_text)?;
    Ok(parse_text(&config_text))
}

pub async fn persist_nodes(
    database: &Database,
    mut source: ImportSource,
    nodes: &[Node],
    name: Option<&str>,
    headers: Option<&std::collections::BTreeMap<String, String>>,
) -> crate::app::Result<ImportSummary> {
    if let Some(name) = name {
        source.name = Some(name.to_string());
    }
    let summary = database
        .import_nodes_with_headers(&source, nodes, headers)
        .await?;
    if let Some(name) = name {
        database
            .set_subscription_name(summary.subscription_id, name)
            .await?;
    }
    Ok(summary)
}

pub fn load_single_node(input: &str) -> crate::app::Result<(ImportSource, Node)> {
    reject_raw_json_config(input)?;

    let mut nodes = parse_text(input).into_iter();
    let Some(node) = nodes.next() else {
        return Err(AppError::NoSupportedConfig);
    };

    if nodes.next().is_some() {
        return Err(AppError::MultipleConfigsForAdd);
    }

    Ok((
        ImportSource {
            kind: SourceKind::RawText,
            value: input.to_string(),
            name: node.name.clone(),
        },
        node,
    ))
}

fn reject_raw_json_config(config_text: &str) -> crate::app::Result<()> {
    if serde_json::from_str::<serde_json::Value>(config_text).is_ok() {
        return Err(AppError::RawJsonImportUnsupported);
    }

    Ok(())
}

pub async fn load_url_list(path: &str) -> crate::app::Result<(ImportSource, Vec<Node>)> {
    if !std::path::Path::new(path).is_file() {
        return Err(AppError::InvalidArgument(
            "--url-list requires a local file of subscription URLs".into(),
        ));
    }
    let (source, bytes) = read_input_async(path).await?;
    let text = decode_or_raw_text(&bytes)?;
    let urls: Vec<_> = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect();
    if urls.iter().any(|url| !looks_like_url(url)) {
        return Err(AppError::InvalidArgument(
            "--url-list accepts only HTTP(S) subscription URLs".into(),
        ));
    }
    let mut nodes = Vec::new();
    for url in urls {
        let (_, body) = read_input_async(url).await?;
        nodes.extend(parse_nodes(&body)?);
    }
    Ok((source, nodes))
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod subscription_tests;
