use clap::Subcommand;

use crate::controllers::operators_list::OperatorsListController;

#[derive(Subcommand)]
pub(crate) enum OperatorsActions {
    List(OperatorsListController),
}
