use clap::Subcommand;

use crate::controllers::{
    memories_add::MemoriesAddController, memories_delete::MemoriesDeleteController,
    memories_list::MemoriesListController,
};

#[derive(Subcommand)]
pub(crate) enum MemoriesActions {
    List(MemoriesListController),
    Add(MemoriesAddController),
    Delete(MemoriesDeleteController),
}
