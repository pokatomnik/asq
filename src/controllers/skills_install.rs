use crate::{controllers::controller::Controller, services::skills::Skills};
use clap::Args;

#[derive(Args, Clone, Debug)]
pub(crate) struct SkillsInstallController {
    #[arg(long = "name", short = 'n', help = "Name of skill")]
    name: String,

    /// URL to Skill markdown
    url: String,
}

impl Controller for SkillsInstallController {
    fn handle(&self) -> anyhow::Result<()> {
        let current_dir = std::env::current_dir()?;
        let skills = Skills::new(current_dir);
        skills.install(self.name.as_str(), self.url.as_str())?;

        println!("Skill installed");

        Ok(())
    }
}
