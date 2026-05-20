use crate::controllers::providers_add::ProvidersAddController;
use crate::controllers::providers_delete::ProvidersDeleteController;
use crate::controllers::providers_list::ProvidersListController;
use clap::Subcommand;

#[derive(Subcommand)]
pub(crate) enum ProvidersActions {
    List(ProvidersListController),
    Add(ProvidersAddController),
    Delete(ProvidersDeleteController),
}
