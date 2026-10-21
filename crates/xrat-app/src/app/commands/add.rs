use crate::app::commands::output;
use crate::app::commands::progress::CliProgress;
use crate::app::{context::AppContext, import};

pub async fn run(context: &AppContext, input: &str) -> crate::app::Result<()> {
    let progress = CliProgress::spinner(true, "adding config");
    let result = async {
        let (source, node) = import::load_single_node(input)?;
        let summary = context.db.import_nodes(&source, &[node]).await?;
        crate::app::Result::Ok((source, summary))
    }
    .await;
    progress.finish_and_clear();
    let (source, summary) = result?;

    print_summary(
        source.kind.as_str(),
        summary.total_configs,
        &context.runtime_paths.database_label,
        &context.runtime_paths.config_path.display().to_string(),
    );
    Ok(())
}

pub(super) fn print_summary(
    source_kind: &str,
    total_configs: i64,
    database_label: &str,
    config_path: &str,
) {
    println!(
        "{}",
        output::success("Added 1 config.", output::color_enabled())
    );
    println!(
        "{}",
        output::format_kv(
            None,
            &[
                ("database", database_label.to_string()),
                ("config", config_path.to_string()),
                ("source", source_kind.to_string()),
                ("total configs", total_configs.to_string()),
            ],
            output::color_enabled(),
        )
    );
}
