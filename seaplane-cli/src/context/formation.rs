use std::collections::HashSet;

use seaplane::api::v1::formations::{
    FormationConfiguration as FormationConfigurationModel, Provider as ProviderModel,
    Region as RegionModel,
};

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
            .values_of("include-flight-plan")
            .unwrap_or_default()
            // Filter out @ strings and inline definitions
            .filter(|f| !(f.starts_with('@') || f.contains('=')))
        {
            // Try to lookup either a partial ID match, or exact NAME match
            if let Some(flight) = flights_db.find_name_or_partial_id(flight) {
                // Look for exact name matches, or partial ID matches and map to their name
                flight_names.push(flight.model.name().to_owned());
            } else {
                // No match
                return Err(no_matching_flight(flight));
            }
        }

        self.name_id = matches
            .value_of("name_id")
            .map(ToOwned::to_owned)
            .unwrap_or_else(generate_formation_name);

        self.grounded = matches.is_present("grounded");
        self.launch = matches.is_present("launch");
        self.cfg_ctx
            .flights
            .extend(flight_names.iter().map(|s| s.to_string()));
        self.cfg_ctx.affinities.extend(
            matches
                .values_of("affinity")
                .unwrap_or_default()
                .map(ToOwned::to_owned),
        );
        self.cfg_ctx.connections.extend(
            matches
                .values_of("connection")
                .unwrap_or_default()
                .map(ToOwned::to_owned),
        );
        self.cfg_ctx.providers_allowed =
            values_t_or_exit!(@into_model matches, "provider", Provider);

        self.cfg_ctx.providers_denied =
            values_t_or_exit!(@into_model matches, "exclude-provider", Provider);
        self.cfg_ctx.regions_allowed = values_t_or_exit!(@into_model matches, "region", Region);
        self.cfg_ctx.regions_denied =
            values_t_or_exit!(@into_model matches, "exclude-region", Region);
        self.cfg_ctx.public_endpoints = values_t_or_exit!(matches, "public-endpoint", Endpoint);
        self.cfg_ctx.formation_endpoints =
            values_t_or_exit!(matches, "formation-endpoint", Endpoint);
        self.cfg_ctx.flight_endpoints = values_t_or_exit!(matches, "flight-endpoint", Endpoint);
        Ok(())
    }

    /// Creates a new seaplane::api::v1::FormationConfiguration from the contained values
    pub fn configuration_model(&self, ctx: &Ctx) -> Result<FormationConfigurationModel> {
        // Create the new Formation model from the CLI inputs
        let mut f_model = FormationConfigurationModel::builder();

        for flight_name in &self.cfg_ctx.flights {
            let flight = ctx
                .db
                .flights
                .find_name(flight_name)
                .ok_or_else(|| no_matching_flight(flight_name))?;
            f_model = f_model.add_flight(flight.model.clone());
        }

        // TODO: clean this up...yuck
        for &item in &self.cfg_ctx.providers_allowed {
            f_model = f_model.add_allowed_provider(item);
        }
        for &item in &self.cfg_ctx.providers_denied {
            f_model = f_model.add_denied_provider(item);
        }
        for &item in &self.cfg_ctx.regions_allowed {
            f_model = f_model.add_allowed_region(item);
        }
        for &item in &self.cfg_ctx.regions_denied {
            f_model = f_model.add_denied_region(item);
        }
        for item in &self.cfg_ctx.public_endpoints {
            f_model = f_model.add_public_endpoint(item.key(), item.value());
        }
        for item in &self.cfg_ctx.flight_endpoints {
            f_model = f_model.add_flight_endpoint(item.key(), item.value());
        }
        #[cfg(feature = "unstable")]
        {
            for item in &self.cfg_ctx.affinities {
                f_model = f_model.add_affinity(item);
            }
            for item in &self.cfg_ctx.connections {
                f_model = f_model.add_connection(item);
            }
            for item in &self.cfg_ctx.formation_endpoints {
                f_model = f_model.add_formation_endpoint(item.key(), item.value());
            }
        }

        // TODO: probably match and check errors
        f_model.build().map_err(Into::into)
    }
}

#[derive(Default, Debug, Clone)]
pub struct FormationCfgCtx {
    /// `String` is a flight name because that's the only thing shared by both local and remote
    pub flights: Vec<String>,
    /// `String` is a flight name because that's the only thing shared by both local and remote
    pub affinities: Vec<String>,
    /// `String` is a flight name because that's the only thing shared by both local and remote
    pub connections: Vec<String>,
    /// Use actual API model since that is ultimately what we want
    pub providers_allowed: HashSet<ProviderModel>,
    /// Use actual API model since that is ultimately what we want
    pub providers_denied: HashSet<ProviderModel>,
    /// Use actual API model since that is ultimately what we want
    pub regions_allowed: HashSet<RegionModel>,
    /// Use actual API model since that is ultimately what we want
    pub regions_denied: HashSet<RegionModel>,
    pub public_endpoints: Vec<Endpoint>,
    pub formation_endpoints: Vec<Endpoint>,
    pub flight_endpoints: Vec<Endpoint>,
}
