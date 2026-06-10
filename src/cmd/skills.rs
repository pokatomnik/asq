use clap::Subcommand;

use crate::controllers::{
    skills_delete::SkillsDeleteController, skills_install::SkillsInstallController,
    skills_list::SkillsListController,
};

#[derive(Subcommand)]
pub(crate) enum SkillsActions {
    List(SkillsListController),
    Install(SkillsInstallController),
    Delete(SkillsDeleteController),
}
