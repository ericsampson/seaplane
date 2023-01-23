pub mod common;
#[cfg(feature = "unstable")]
mod configuration;
#[cfg(feature = "unstable")]
mod container_stats;
mod delete;
mod fetch;
mod land;
mod launch;
mod list;
#[cfg(feature = "unstable")]
mod load_balance;
mod plan;
mod status;
#[cfg(feature = "unstable")]
mod template;

use clap::{ArgMatches, Command};
pub use common::{Provider, Region};
pub use plan::SeaplaneFormationPlanArgMatches;

#[cfg(feature = "unstable")]
use self::{
    configuration::SeaplaneFormationConfiguration,
    container_stats::SeaplaneFormationContainerStatistics,
    load_balance::SeaplaneFormationLoadBalance, template::SeaplaneFormationTemplate,
};
pub use self::{
    delete::SeaplaneFormationDelete, fetch::SeaplaneFormationFetch, land::SeaplaneFormationLand,
    launch::SeaplaneFormationLaunch, list::SeaplaneFormationList, plan::SeaplaneFormationPlan,
    status::SeaplaneFormationStatus,
};
use crate::{cli::CliCommand, error::Result, Ctx};

/// Allows eliding `registry` but otherwise just proxies parsing to ImageReference
pub fn str_to_image_ref(registry: &str, image_str: &str) -> Result<ImageReference> {
    match image_str.parse::<ImageReference>() {
        Ok(ir) => Ok(ir),
        Err(ImageReferenceError::ErrDomainInvalidFormat(_)) => {
            let ir: ImageReference = format!("{registry}/{image_str}").parse()?;
            Ok(ir)
        }
        Err(e) => Err(CliError::from(e)),
    }
}

#[derive(Copy, Clone, Debug)]
pub struct SeaplaneFormation;

impl SeaplaneFormation {
    pub fn command() -> Command {
        #[cfg_attr(not(feature = "unstable"), allow(unused_mut))]
        let mut app = Command::new("formation")
            .about(
                "Operate on local Formations Plans and remote Formation Instances of those Plans",
            )
            .subcommand_required(true)
            .arg_required_else_help(true)
            .subcommand(SeaplaneFormationPlan::command())
            .subcommand(SeaplaneFormationDelete::command())
            .subcommand(SeaplaneFormationFetch::command())
            .subcommand(SeaplaneFormationLand::command())
            .subcommand(SeaplaneFormationLaunch::command())
            .subcommand(SeaplaneFormationList::command())
            .subcommand(SeaplaneFormationStatus::command());

        #[cfg(feature = "unstable")]
        {
            app = app
                .subcommand(SeaplaneFormationConfiguration::command())
                .subcommand(SeaplaneFormationContainerStatistics::command())
                .subcommand(SeaplaneFormationLoadBalance::command())
                .subcommand(SeaplaneFormationTemplate::command())
        }

        app
    }
}

impl CliCommand for SeaplaneFormation {
    fn next_subcmd<'a>(
        &self,
        matches: &'a ArgMatches,
    ) -> Option<(Box<dyn CliCommand>, &'a ArgMatches)> {
        match &matches.subcommand() {
            Some(("plan", m)) => Some((Box::new(SeaplaneFormationPlan), m)),
            Some(("delete", m)) => Some((Box::new(SeaplaneFormationDelete), m)),
            Some(("fetch-remote", m)) => Some((Box::new(SeaplaneFormationFetch), m)),
            Some(("land", m)) => Some((Box::new(SeaplaneFormationLand), m)),
            Some(("launch", m)) => Some((Box::new(SeaplaneFormationLaunch), m)),
            Some(("list", m)) => Some((Box::new(SeaplaneFormationList), m)),
            Some(("status", m)) => Some((Box::new(SeaplaneFormationStatus), m)),
            #[cfg(feature = "unstable")]
            Some(("configuration", m)) => Some((Box::new(SeaplaneFormationConfiguration), m)),
            #[cfg(feature = "unstable")]
            Some(("container-statistics", m)) => {
                Some((Box::new(SeaplaneFormationContainerStatistics), m))
            }
            #[cfg(feature = "unstable")]
            Some(("load-balance", m)) => Some((Box::new(SeaplaneFormationLoadBalance), m)),
            #[cfg(feature = "unstable")]
            Some(("template", m)) => Some((Box::new(SeaplaneFormationTemplate), m)),
            _ => None,
        }
    }

    fn update_ctx(&self, _matches: &ArgMatches, _ctx: &mut Ctx) -> Result<()> { Ok(()) }
}
