use std::collections::HashSet;

use seaplane::api::compute::v2::Formation as FormationModel;

use crate::{
    cli::{cmds::formation::SeaplaneFormationPlanArgMatches, Provider, Region},
    context::Ctx,
    error::{CliError, CliErrorKind, Context, Result},
    ops::{flight::Flights, formation::Endpoint, generate_formation_name},
    printer::Color,
};

fn no_matching_flight(flight: &str) -> CliError {
    CliErrorKind::NoMatchingItem(flight.to_string())
        .into_err()
        .context("(hint: create the Flight Plan with '")
        .with_color_context(|| (Color::Green, format!("seaplane flight plan {flight}")))
        .context("')\n")
        .context("(hint: or try fetching remote definitions with '")
        .color_context(Color::Green, "seaplane formation fetch-remote")
        .context("')\n")
}

/// Represents the "Source of Truth" i.e. it combines all the CLI options, ENV vars, and config
/// values into a single structure that can be used later to build models for the API or local
/// structs for serializing
///
/// A somewhat counter-intuitive thing about Formations and their models is the there is no
/// "Formation Model" only a "Formation Configuration Model" This is because a "Formation" so to
/// speak is really just a named collection of configurations and info about their traffic
/// weights/activation statuses.
// TODO: we may not want to derive this we implement circular references
#[derive(Debug, Clone)]
pub struct FormationCtx {
    pub name_id: String,
    pub launch: bool,
    pub remote: bool,
    pub local: bool,
    pub grounded: bool,
    pub recursive: bool,
    // TODO: make multiple possible
    pub cfg_ctx: FormationCfgCtx,
}

impl Default for FormationCtx {
    fn default() -> Self {
        Self {
            name_id: generate_formation_name(),
            launch: false,
            cfg_ctx: FormationCfgCtx::default(),
            remote: false,
            local: true,
            grounded: false,
            recursive: false,
        }
    }
}

impl FormationCtx {
    /// `flight` is the name of the argument for the Flight's name/id
    pub fn update_from_formation_plan(
        &mut self,
        matches: &SeaplaneFormationPlanArgMatches,
        flights_db: &Flights,
    ) -> Result<()> {
        let matches = matches.0;
        // TODO: check if "all" was used along with another value within regions/providers and err

        let mut flight_names = Vec::new();

        // Translate the flight NAME|ID into a NAME
        for flight in matches
            .get_many::<String>("include-flight-plan")
            .unwrap_or_default()
            // Filter out @ strings and inline definitions
            .filter(|f| !(f.starts_with('@') || f.contains('=')))
        {
            // Try to lookup either a partial ID match, or exact NAME match
            if let Some(flight) = flights_db.find_name_or_partial_id(flight) {
                // Look for exact name matches, or partial ID matches and map to their name
                flight_names.push(flight.model.name().to_owned());
            } else {
                #[cfg(not(any(feature = "ui_tests", feature = "semantic_ui_tests",)))]
                {
                    // No match
                    return Err(no_matching_flight(flight));
                }
            }
        }

        self.name_id = matches
            .get_one::<String>("name_id")
            .map(ToOwned::to_owned)
            .unwrap_or_else(generate_formation_name);

        self.grounded = matches.get_flag("grounded");
        self.launch = matches.get_flag("launch");
        self.cfg_ctx
            .flights
            .extend(flight_names.iter().map(|s| s.to_string()));
        Ok(())
    }

    /// Creates a new seaplane::api::compute::v2::Formation from the contained values
    pub fn configuration_model(&self, ctx: &Ctx) -> Result<FormationModel> {
        // Create the new Formation model from the CLI inputs
        let mut f_model = FormationModel::builder();

        for flight_name in &self.cfg_ctx.flights {
            let flight = ctx
                .db
                .flights
                .find_name(flight_name)
                .ok_or_else(|| no_matching_flight(flight_name))?;
            f_model = f_model.add_flight(flight.model.clone());
        }

        // TODO: probably match and check errors
        f_model.build().map_err(Into::into)
    }
}

#[derive(Default, Debug, Clone)]
pub struct FormationCfgCtx {
    /// `String` is a flight name because that's the only thing shared by both local and remote
    pub flights: Vec<String>,
}
