use clap::Subcommand;

use crate::controllers::memories_add::MemoriesAddController;
use crate::controllers::memories_delete::MemoriesDeleteController;
use crate::controllers::memories_list::MemoriesListController;

#[derive(Subcommand)]
pub(crate) enum MemoriesActions {
    List(MemoriesListController),
    Add(MemoriesAddController),
    Delete(MemoriesDeleteController),
}
