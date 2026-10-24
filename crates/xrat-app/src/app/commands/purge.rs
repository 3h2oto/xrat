use crate::app::commands::output;
use crate::app::context::AppContext;
use crate::cli::PurgeArgs;

pub async fn run(context: &AppContext, args: &PurgeArgs) -> crate::app::Result<()> {
    let pending = context.services().lifecycle.count_deleted().await?;
    if !confirm(pending, args.yes)? {
        return Ok(());
    }
    let purged = context.services().lifecycle.purge().await?;
    print_count(purged);
    Ok(())
}

pub(super) fn confirm(pending: i64, yes: bool) -> crate::app::Result<bool> {
    if pending == 0 {
        println!(
            "{}",
            output::notice("No soft-deleted configs to purge.", output::color_enabled())
        );
        return Ok(false);
    }

    if !yes
        && !output::confirm(format!(
            "Permanently delete {pending} soft-deleted config(s)?"
        ))?
    {
        println!("{}", output::notice("Aborted.", output::color_enabled()));
        return Ok(false);
    }

    Ok(true)
}

pub(super) fn print_count(purged: u64) {
    println!(
        "{}",
        output::success(
            format!("Purged {purged} config(s)."),
            output::color_enabled()
        )
    );
}
