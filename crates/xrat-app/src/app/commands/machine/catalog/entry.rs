use crate::app::services::machine_service::{CatalogPayload, CatalogRequest, MachineClient};
use crate::app::{AppError, Result};
use crate::cli::Command;

pub(crate) async fn run(client: &MachineClient, command: &Command) -> Result<()> {
    match command {
        Command::Tun(args) => super::tun::run(client,args).await,
        Command::Test(args) => super::probes::run(client, args).await,
        Command::Logs(args) => super::logs::run(client, args).await,
        Command::List(args) => super::list::run(client, args).await,
        Command::Import(args) => super::imports::import(client, args).await,
        Command::Update(args) => super::imports::update(client, args).await,
        Command::Add(args) => super::imports::add(client, &args.input).await,
        Command::Enable(args) => super::lifecycle::toggle(client, &args.id, true).await,
        Command::Disable(args) => super::lifecycle::toggle(client, &args.id, false).await,
        Command::Delete(args) => super::lifecycle::delete(client, args).await,
        Command::Purge(args) => super::lifecycle::purge(client, args).await,
        Command::Restore(args) => super::lifecycle::restore(client, &args.id).await,
        Command::Show(args) => super::lifecycle::show(client, args).await,
        Command::Connect(args) => super::runtime::connect(client, args).await,
        Command::Disconnect(args) => super::runtime::disconnect(client, args).await,
        Command::Rotate(args) => super::rotation::run(client, args).await,
        Command::Status(args) => super::runtime::status(client, args).await,
        _ => Err(AppError::InvalidArgument(
            "this operation is not yet available through the machine service; the user profile was not opened".into(),
        )),
    }
}

pub(super) async fn subscription(
    client: &MachineClient,
    reference: &str,
) -> Result<xrat_db::SubscriptionRecord> {
    client.subscription(reference).await
}

pub(super) async fn profile_labels(client: &MachineClient) -> Result<(String, String)> {
    match client.catalog(CatalogRequest::Profile {}).await? {
        CatalogPayload::Profile {
            database_label,
            config_path,
        } => Ok((database_label, config_path)),
        _ => Err(unexpected()),
    }
}

pub(super) fn unexpected() -> AppError {
    AppError::InvalidArgument("machine service returned an unexpected catalog result".into())
}
