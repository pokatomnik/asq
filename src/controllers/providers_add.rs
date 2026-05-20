use clap::Args;

use crate::controllers::controller::Controller;

#[derive(Args, Clone, Debug)]
pub(crate) struct ProvidersAddController {}

impl Controller for ProvidersAddController {
    fn handle(&self) -> anyhow::Result<()> {
        todo!()
    }
}
