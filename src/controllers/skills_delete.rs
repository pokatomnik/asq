use crate::{controllers::controller::Controller, services::skills::Skills};
use clap::Args;

#[derive(Args, Clone, Debug)]
pub(crate) struct SkillsDeleteController;

impl Controller for SkillsDeleteController {
    fn handle(&self) -> anyhow::Result<()> {
        let current_dir = std::env::current_dir()?;
        let skills = Skills::new(current_dir);

        let Ok(_) = skills.delete_interactive() else {
            println!("No skills to delete");
            return Ok(());
        };

        println!("Skill removed");

        Ok(())
    }
}
