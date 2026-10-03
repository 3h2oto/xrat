use std::collections::BTreeSet;

use crate::cli::ScanArgs;

pub(super) fn collect_ips(args: &ScanArgs) -> crate::app::Result<Vec<String>> {
    let mut dedup = BTreeSet::new();
    for ip in &args.ips {
        let ip = ip.trim();
        if !ip.is_empty() {
            dedup.insert(ip.to_string());
        }
    }

    if let Some(path) = &args.file {
        let input = std::fs::read_to_string(path)?;
        for line in input.lines() {
            let ip = line.trim();
            if !ip.is_empty() {
                dedup.insert(ip.to_string());
            }
        }
    }

    Ok(dedup.into_iter().collect())
}

#[cfg(test)]
mod tests;
