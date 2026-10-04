use crate::app::AppError;
use crate::app::input::source::{read_input, read_input_async};
use xrat_config::parse_text;
use xrat_db::ImportSource;
use xrat_db::SourceKind;
use xrat_db::{Database, ImportSummary};
use xrat_model::Node;
use xrat_support::decode::decode_or_raw_text;
use xrat_support::url::looks_like_url;

pub fn load_nodes(input: &str) -> crate::app::Result<(ImportSource, Vec<Node>)> {
    let (source, input_data) = read_input(input)?;
    let config_text = decode_or_raw_text(&input_data)?;
    reject_raw_json_config(&config_text)?;
    let normalized_text = expand_url_list(&config_text)?;

    Ok((source, parse_text(&normalized_text)))
}

pub async fn load_nodes_async(input: &str) -> crate::app::Result<(ImportSource, Vec<Node>)> {
    let (source, input_data) = read_input_async(input).await?;
    let config_text = decode_or_raw_text(&input_data)?;
    reject_raw_json_config(&config_text)?;
    let normalized_text = expand_url_list_async(&config_text).await?;

    Ok((source, parse_text(&normalized_text)))
}

pub async fn persist_nodes(
    database: &Database,
    mut source: ImportSource,
    nodes: &[Node],
    name: Option<&str>,
) -> crate::app::Result<ImportSummary> {
    if let Some(name) = name {
        source.name = Some(name.to_string());
    }
    let summary = database.import_nodes(&source, nodes).await?;
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

fn expand_url_list(input: &str) -> crate::app::Result<String> {
    let mut collected = Vec::new();
    let mut saw_url = false;

    for line in input.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if looks_like_url(trimmed) {
            saw_url = true;
            let (_, body) = read_input(trimmed)?;
            collected.push(decode_or_raw_text(&body)?);
        } else {
            collected.push(trimmed.to_string());
        }
    }

    if saw_url {
        Ok(collected.join("\n"))
    } else {
        Ok(input.to_string())
    }
}

async fn expand_url_list_async(input: &str) -> crate::app::Result<String> {
    let mut collected = Vec::new();
    let mut saw_url = false;

    for line in input.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if looks_like_url(trimmed) {
            saw_url = true;
            let (_, body) = read_input_async(trimmed).await?;
            collected.push(decode_or_raw_text(&body)?);
        } else {
            collected.push(trimmed.to_string());
        }
    }

    if saw_url {
        Ok(collected.join("\n"))
    } else {
        Ok(input.to_string())
    }
}

#[cfg(test)]
mod tests;
