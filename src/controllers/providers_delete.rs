use clap::Args;

use crate::controllers::controller::Controller;

#[derive(Args, Clone, Debug)]
pub(crate) struct ProvidersDeleteController {}

impl Controller for ProvidersDeleteController {
    fn handle(&self) -> anyhow::Result<()> {
        todo!()
    }
}
