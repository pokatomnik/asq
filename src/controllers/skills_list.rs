use crate::{controllers::controller::Controller, services::skills::Skills};
use clap::Args;

#[derive(Args, Clone, Debug)]
pub(crate) struct SkillsListController;

impl Controller for SkillsListController {
    fn handle(&self) -> anyhow::Result<()> {
        let current_dir = std::env::current_dir()?;
        let skills = Skills::new(current_dir);
        let Some(contents) = skills.get_contents_interactive() else {
            println!("You have no skills installed or cannot read them");
            return Ok(());
        };

        println!("{}", contents.as_str());

        Ok(())
    }
}
