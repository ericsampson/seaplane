use std::{
    collections::HashSet,
    io::Write,
    path::{Path, PathBuf},
};

use seaplane::api::compute::v2::{Flight as FlightModel, Formation as FormationModel};
use serde::{Deserialize, Serialize};
use tabwriter::TabWriter;
use uuid::Uuid;

use crate::{
    context::Ctx,
    error::{CliError, Result},
    fs::{FromDisk, ToDisk},
    ops::Id,
    printer::Output,
};

// TODO: Change out the Vecs for HashMaps where the key is an ID
/// This struct represents a Local Formation. I.e. one the user can interact with on the CLI and can
/// be (de)serialized locally.
///
/// A somewhat counter-intuitive thing about "Formations" and their models is the there is no
/// "Formation Model" only a "Formation Configuration Model" This is because a "Formation" so to
/// speak is really just a named collection of configurations and info about their traffic
/// weights/activation statuses.
#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct Formations {
    /// A list of "Formation"s
    #[serde(default)]
    pub formations: Vec<Formation>,
}

impl Formations {
    /// Extend and de-duplicate from another set of Formations
    pub fn extend(&mut self, rhs: &Formations) {
        self.formations.extend_from_slice(&rhs.formations);
        self.formations.dedup_by(|l, r| l.id == r.id);
        self.configurations.extend_from_slice(&rhs.configurations);
        self.configurations.dedup_by(|l, r| l.id == r.id);
    }

    pub fn get_configuration_by_uuid(&self, uuid: Uuid) -> Option<&FormationConfiguration> {
        self.configurations
            .iter()
            .find(|fc| fc.remote_id == Some(uuid))
    }

    pub fn remote_names(&self) -> Vec<&str> {
        self.formations
            .iter()
            .filter(|f| !f.in_air.is_empty() || !f.grounded.is_empty())
            .filter_map(|f| f.name.as_deref())
            .collect()
    }

    pub fn has_flight(&self, flight: &str) -> bool {
        self.configurations
            .iter()
            .any(|fc| fc.model.flights().iter().any(|f| f.name() == flight))
    }

    pub fn formations(&self) -> impl Iterator<Item = &Formation> { self.formations.iter() }

    /// Returns the removed FormationConfiguration by ID or None if there was no match
    ///
    /// DANGER: this will invalidate any previously held indices after the removed item
    pub fn remove_configuration(&mut self, id: &Id) -> Option<FormationConfiguration> {
        if let Some(idx) = self.configuration_index_of_id(id) {
            return Some(self.configurations.swap_remove(idx));
        }
        None
    }

    // TODO: this should go away once we're not working with indices anymore
    pub fn get_formation(&self, idx: usize) -> Option<&Formation> { self.formations.get(idx) }

    // TODO: this should go away once we're not working with indices anymore
    pub fn get_formation_mut(&mut self, idx: usize) -> Option<&mut Formation> {
        self.formations.get_mut(idx)
    }

    /// Either updates a matching local Formation Configurations, or creates a new one. Returns the
    /// existing ID of the config that was updated if any
    pub fn update_or_create_configuration(&mut self, cfg: FormationConfiguration) -> Option<Id> {
        let has_matching_uuid = self
            .configurations
            .iter()
            .any(|c| c.remote_id == cfg.remote_id);
        if let Some(old_cfg) = self
            .configurations
            .iter_mut()
            .find(|c| c.model == cfg.model && (c.remote_id.is_none() && !has_matching_uuid))
        {
            // This should have come from the API and thus requires a UUID
            old_cfg.remote_id = Some(cfg.remote_id.unwrap());
            Some(old_cfg.id)
        } else if self.configurations.iter().any(|c| c.eq_without_id(&cfg)) {
            None
        } else {
            self.configurations.push(cfg);
            None
        }
    }

    /// Either updates a matching local Formations by replacing the local IDs, or creates a new
    /// one. Returns NEW Formations IDs
    pub fn update_or_create_formation(&mut self, formation: Formation) -> Option<Id> {
        if let Some(f) = self
            .formations
            .iter_mut()
            .find(|f| f.name == formation.name)
        {
            f.in_air = formation.in_air;
            f.grounded = formation.grounded;
            f.local = formation.local;
            None
        } else {
            let id = formation.id;
            self.formations.push(formation);
            Some(id)
        }
    }

    /// Returns true if there is a Formation with the given name
    pub fn contains_name(&self, name: &str) -> bool {
        self.formations
            .iter()
            .any(|f| f.name.as_deref() == Some(name))
    }

    /// Removes an exact name match, returning the removed Formation or None if nothing matched.
    ///
    /// DANGER: this will invalidate any previously held indices after the removed item
    pub fn remove_name(&mut self, name: &str) -> Option<Formation> {
        cli_traceln!("Removing Formation {name} from local state");
        if let Some(idx) = self.formation_index_of_name(name) {
            return Some(self.formations.swap_remove(idx));
        }

        None
    }

    // TODO: this should go away once we're not working with indices anymore
    /// Returns the index of an exact name match
    pub fn formation_index_of_name(&self, name: &str) -> Option<usize> {
        cli_traceln!("Searching local DB for index of Formation Plan {name}");
        self.formations
            .iter()
            .enumerate()
            .find(|(_, f)| f.name.as_deref() == Some(name))
            .map(|(i, _)| i)
    }

    pub fn configuration_index_of_id(&self, id: &Id) -> Option<usize> {
        cli_traceln!("Searching for index of Configuration ID {id}");
        self.configurations
            .iter()
            .enumerate()
            .find(|(_, c)| &c.id == id)
            .map(|(i, _)| i)
    }

    // TODO: this should go away once we're not working with indices anymore
    /// Returns all indices of an exact name or partial ID match
    pub fn formation_indices_of_matches(&self, name: &str) -> Vec<usize> {
        cli_traceln!("Searching local DB for exact matches of Formation Plan {name}");
        self.formations
            .iter()
            .enumerate()
            .filter(|(_, f)| f.name.as_deref() == Some(name) || f.id.to_string().starts_with(name))
            .map(|(i, _)| i)
            .collect()
    }

    // TODO: this should go away once we're not working with indices anymore
    /// Returns all indices of a partial name or ID match
    pub fn formation_indices_of_left_matches(&self, name: &str) -> Vec<usize> {
        cli_traceln!("Searching local DB for partial matches of Formation Plan {name}");
        self.formations
            .iter()
            .enumerate()
            .filter(|(_, f)| {
                f.name
                    .as_deref()
                    .map(|n| n.starts_with(name))
                    .unwrap_or(false)
                    || f.id.to_string().starts_with(name)
            })
            .map(|(i, _)| i)
            .collect()
    }

    // TODO: this should go away once we're not working with indices anymore
    /// Removes all indices
    pub fn remove_formation_indices(&mut self, indices: &[usize]) -> Vec<Formation> {
        cli_traceln!("Removing indexes {indices:?} from local state");
        // TODO: There is probably a much more performant way to remove a bunch of times from a Vec
        // but we're talking such a small number of items this should never matter.

        indices
            .iter()
            .enumerate()
            .map(|(i, idx)| self.formations.remove(idx - i))
            .collect()
    }

    /// Removes the given flight from all formations that reference it
    pub fn remove_flight(&mut self, flight: &str) {
        self.configurations.iter_mut().for_each(|cfg| {
            cfg.model.remove_flight(flight);
        });
    }
}

// used in state v1 only
impl FromDisk for Formations {}
impl ToDisk for Formations {}

impl Output for Formations {
    fn print_json(&self, _ctx: &Ctx) -> Result<()> {
        cli_println!("{}", serde_json::to_string(self)?);

        Ok(())
    }

    fn print_table(&self, _ctx: &Ctx) -> Result<()> {
        let buf = Vec::new();
        let mut tw = TabWriter::new(buf);
        writeln!(tw, "LOCAL ID\tNAME\tLOCAL\tDEPLOYED")?;
        for formation in &self.formations {
            let local = formation.local.len();
            let in_air = formation.in_air.len();
            let grounded = formation.grounded.len();
            let total = formation
                .in_air
                .union(
                    &formation
                        .grounded
                        .union(&formation.local)
                        .copied()
                        .collect(),
                )
                .count();

            writeln!(
                tw,
                "{}\t{}\t{}\t{}",
                &formation.id.to_string()[..8], // TODO: make sure length is not ambiguous
                formation.name.as_deref().unwrap_or_default(),
                local,
                in_air,
            )?;
        }
        tw.flush()?;

        cli_println!(
            "{}",
            String::from_utf8_lossy(
                &tw.into_inner()
                    .map_err(|_| CliError::bail("IO flush error"))?
            )
        );

        Ok(())
    }
}

// TODO: move ID to the key of a HashMap
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Formation {
    pub id: Id,
    pub name: Option<String>,
    pub local: HashSet<Id>,
    pub in_air: HashSet<Id>,
}

impl Formation {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            id: Id::new(),
            name: Some(name.into()),
            local: HashSet::new(),
            in_air: HashSet::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.local.is_empty() && self.in_air.is_empty() && self.grounded.is_empty()
    }

    /// Replaces all occurrences of `old_id` with `new_id`
    pub fn replace_id(&mut self, old_id: &Id, new_id: Id) {
        if self.local.remove(old_id) {
            self.local.insert(new_id);
        }
        if self.in_air.remove(old_id) {
            self.in_air.insert(new_id);
        }
        if self.grounded.remove(old_id) {
            self.grounded.insert(new_id);
        }
    }

    /// Returns the Formation Configuration IDs that are neither Grounded (Inactive) or In Air
    /// (active)
    pub fn local_only_configs(&self) -> Vec<Id> {
        self.local
            .difference(&self.in_air.union(&self.grounded).copied().collect())
            .copied()
            .collect()
    }

    /// Returns the Formation Configuration IDs that are either Grounded (Inactive) or Local
    pub fn local_or_grounded_configs(&self) -> Vec<Id> {
        self.local
            .iter()
            .chain(self.grounded.iter())
            .copied()
            .collect::<HashSet<_>>()
            .difference(&self.in_air)
            .copied()
            .collect()
    }

    /// Returns a deduplicated union of all the configuration IDs.
    pub fn configs(&self) -> Vec<Id> {
        self.local
            .iter()
            .chain(self.in_air.iter().chain(self.grounded.iter()))
            .copied()
            .collect()
    }
}

/// Wraps the [`FormationConfiguration`] model adding a local ID and the UUID associated
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Formation {
    pub id: Id,
    pub model: FormationModel,
}

impl FormationConfiguration {
    pub fn new(model: FormationConfigurationModel) -> Self {
        Self { id: Id::new(), remote_id: None, model }
    }

    pub fn with_uuid(uuid: Uuid, model: FormationConfigurationModel) -> Self {
        Self { id: Id::new(), remote_id: Some(uuid), model }
    }

    pub fn get_flight(&self, flight: &str) -> Option<&FlightModel> {
        self.model.flights().iter().find(|f| f.name() == flight)
    }

    /// Performs equality check without consider the local ID
    pub fn eq_without_id(&self, other: &Self) -> bool {
        self.remote_id == other.remote_id && self.model == other.model
    }
}

// Possible Symbols?: ◯ ◉ ◍ ◐ ● ○ ◯
const SYM: char = '◉';

#[derive(Debug, Eq, PartialEq, Clone, Serialize)]
pub struct FormationStatus {
    name: String,
    status: OpStatus,
    configurations: FormationConfigStatuses,
}

#[derive(Debug, Default, Eq, PartialEq, Clone, Serialize)]
#[serde(transparent)]
pub struct FormationConfigStatuses {
    inner: Vec<FormationConfigStatus>,
}

#[derive(Debug, Default, Eq, PartialEq, Clone, Serialize)]
pub struct FormationConfigStatus {
    status: OpStatus,
    uuid: Uuid,
    flights: FlightStatuses,
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Serialize)]
#[serde(transparent)]
pub struct FlightStatuses {
    inner: Vec<FlightStatus>,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub struct FlightStatus {
    name: String,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, strum::EnumString, Serialize)]
#[strum(ascii_case_insensitive, serialize_all = "lowercase")]
pub enum OpStatus {
    Up,
    Down,
    Degraded,
    Starting,
}

impl OpStatus {
    pub fn worse_only(&mut self, other: Self) {
        use OpStatus::*;
        match self {
            Up => match other {
                Down => *self = Down,
                Degraded => *self = Degraded,
                Starting => *self = Starting,
                _ => (),
            },
            Down => (),
            Degraded => {
                if other == Down {
                    *self = Down;
                }
            }
            Starting => match other {
                Down => *self = Down,
                Degraded => *self = Degraded,
                _ => (),
            },
        }
    }

    /// Prints the SYM character color coded to the current status
    pub fn print_sym(self) {
        match self {
            OpStatus::Up => cli_print!(@Green, "{SYM}"),
            OpStatus::Down => cli_print!(@Red, "{SYM}"),
            OpStatus::Degraded | OpStatus::Starting => cli_print!(@Yellow, "{SYM}"),
        }
    }

    /// Prints string version of self color coded to the current status
    pub fn print(self) {
        match self {
            OpStatus::Up => cli_print!(@Green, "UP"),
            OpStatus::Down => cli_print!(@Red, "DOWN"),
            OpStatus::Degraded => cli_print!(@Yellow, "DEGRADED"),
            OpStatus::Starting => cli_print!(@Yellow, "STARTING"),
        }
    }

    /// Prints a string color coded to the current status
    pub fn print_msg(self, msg: &str) {
        match self {
            OpStatus::Up => cli_print!(@Green, "{msg}"),
            OpStatus::Down => cli_print!(@Red, "{msg}"),
            OpStatus::Degraded | OpStatus::Starting => cli_print!(@Yellow, "{msg}"),
        }
    }
}

impl Default for OpStatus {
    fn default() -> Self { OpStatus::Starting }
}

impl FormationStatus {
    /// Create a new FormationStatus from a given Formation name.
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            status: OpStatus::default(),
            configurations: FormationConfigStatuses::default(),
        }
    }

    pub fn update_status(&mut self) {
        let mut status = OpStatus::Up;
        for cfg in self.configurations.inner.iter_mut() {
            cfg.update_status();
            status.worse_only(cfg.status);
        }
        self.status = status;
    }
}

impl FlightStatus {
    pub fn new<S: Into<String>>(name: S) -> Self { Self { name: name.into() } }

    #[allow(unused_assignments)]
    pub fn get_status(&self) -> OpStatus {
        let mut status = OpStatus::Up;
        if self.running >= self.minimum {
            status = OpStatus::Up;
        } else {
            status = OpStatus::Degraded;
        }
        if self.running == 0 && self.starting > 0 {
            status = OpStatus::Degraded;
        }
        if self.errored > 0 {
            // TODO even if running > minimum?
            status = OpStatus::Degraded;
        }
        status
    }
}

impl FormationConfigStatuses {
    pub fn add_running_flight<S: Into<String>>(
        &mut self,
        uuid: Uuid,
        name: S,
        min: u64,
        max: Option<u64>,
    ) {
        if let Some(cfg) = self.inner.iter_mut().find(|cfg| cfg.uuid == uuid) {
            cfg.flights.add_running(name, min, max)
        } else {
            let mut fs = FlightStatuses::default();

            fs.add_running(name, min, max);
            self.inner
                .push(FormationConfigStatus { status: OpStatus::Starting, uuid, flights: fs })
        }
    }
    pub fn add_starting_flight<S: Into<String>>(
        &mut self,
        uuid: Uuid,
        name: S,
        min: u64,
        max: Option<u64>,
    ) {
        if let Some(cfg) = self.inner.iter_mut().find(|cfg| cfg.uuid == uuid) {
            cfg.flights.add_starting(name, min, max)
        } else {
            let mut fs = FlightStatuses::default();

            fs.add_starting(name, min, max);
            self.inner
                .push(FormationConfigStatus { status: OpStatus::Starting, uuid, flights: fs })
        }
    }
    pub fn add_stopped_flight<S: Into<String>>(
        &mut self,
        uuid: Uuid,
        name: S,
        error: bool,
        min: u64,
        max: Option<u64>,
    ) {
        if let Some(cfg) = self.inner.iter_mut().find(|cfg| cfg.uuid == uuid) {
            cfg.flights.add_stopped(name, error, min, max)
        } else {
            let mut fs = FlightStatuses::default();

            fs.add_stopped(name, error, min, max);
            self.inner
                .push(FormationConfigStatus { status: OpStatus::Starting, uuid, flights: fs })
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool { self.inner.is_empty() }

    #[inline]
    pub fn len(&self) -> usize { self.inner.len() }
}

impl FormationConfigStatus {
    pub fn update_status(&mut self) {
        let mut status = OpStatus::Up;
        for flight in self.flights.inner.iter() {
            status.worse_only(flight.get_status());
        }
        self.status = status;
    }

    pub fn print_pretty(&self, last: bool) {
        // Chars we'll need: │ ├ ─ └
        if self.flights.is_empty() {
            return;
        }
        if last {
            cli_print!("└─");
        } else {
            cli_print!("├─");
        }
        self.status.print_sym();
        cli_print!(" Configuration {}: ", self.uuid);
        self.status.print();
        cli_println!("");

        if self.flights.is_empty() {
            return;
        }
        let prefix = if last { "  " } else { "│ " };
        cli_println!("{prefix}│");
        // Unfortunately we can't use tabwriter here as we can't color the symbol with that. So we
        // just manually calculate the spaces since it's only a few fields anyways. We also assume
        // the numbered fields aren't going to be higher than 99999999999 and if they are we most
        // likely have other problems.
        macro_rules! nspaces {
            ($n:expr, $w:expr) => {{
                nspaces!(($w.chars().count() + 4) - $n.to_string().chars().count())
            }};
            ($n:expr) => {{
                let mut spaces = String::with_capacity($n);
                for _ in 0..$n {
                    spaces.push(' ');
                }
                spaces
            }};
        }
        let longest_flight_name = self
            .flights
            .inner
            .iter()
            .map(|f| f.name.len())
            .max()
            .unwrap();
        let total_slot_size = std::cmp::max(longest_flight_name, 10);
        let spaces_after_flight = total_slot_size - 6; // 6 = FLIGHT
        cli_println!(
            "{prefix}│   FLIGHT{}RUNNING    EXITED    ERRORED    STARTING    MIN / MAX",
            nspaces!(spaces_after_flight)
        );
        for (i, flight) in self.flights.inner.iter().enumerate() {
            if i == self.flights.inner.len() - 1 {
                cli_print!("{prefix}└─");
            } else {
                cli_print!("{prefix}├─");
            }
            self.status.print_sym();

            let name = &flight.name;
            let running = flight.running;
            let exited = flight.exited;
            let errored = flight.errored;
            let starting = flight.starting;
            let minimum = flight.minimum;
            let maximum = if let Some(maximum) = flight.maximum {
                format!("{maximum}")
            } else {
                "AUTO".to_string()
            };

            let s_after_name = nspaces!(total_slot_size - name.len());
            let s_after_running = nspaces!(running, "RUNNING");
            let s_after_exited = nspaces!(exited, "EXITED");
            let s_after_errored = nspaces!(errored, "ERRORED");
            let s_after_starting = nspaces!(starting, "STARTING");

            cli_println!(" {name}{s_after_name}{running}{s_after_running}{exited}{s_after_exited}{errored}{s_after_errored}{starting}{s_after_starting}{minimum} / {maximum}");
        }
        if last {
            cli_println!("");
        } else {
            cli_println!("│");
        }
    }
}

impl FlightStatuses {
    pub fn add_running<S: Into<String>>(&mut self, name: S, minimum: u64, maximum: Option<u64>) {
        let name = name.into();
        if let Some(f) = self.inner.iter_mut().find(|f| f.name == name) {
            f.running += 1;
        } else {
            self.inner.push(FlightStatus { name })
        }
    }

    pub fn add_stopped<S: Into<String>>(
        &mut self,
        name: S,
        error: bool,
        minimum: u64,
        maximum: Option<u64>,
    ) {
        let name = name.into();
        if let Some(f) = self.inner.iter_mut().find(|f| f.name == name) {
            if error {
                f.errored += 1;
            } else {
                f.exited += 1;
            }
        } else {
            self.inner.push(FlightStatus { name })
        }
    }

    pub fn add_starting<S: Into<String>>(&mut self, name: S, minimum: u64, maximum: Option<u64>) {
        let name = name.into();
        if let Some(f) = self.inner.iter_mut().find(|f| f.name == name) {
            f.starting += 1;
        } else {
            self.inner.push(FlightStatus { name })
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool { self.inner.is_empty() }
}

impl Output for FormationStatus {
    fn print_json(&self, _ctx: &Ctx) -> Result<()> {
        cli_println!("{}", serde_json::to_string(self)?);

        Ok(())
    }

    fn print_table(&self, _ctx: &Ctx) -> Result<()> {
        // Chars we'll need: │ ├ ─ └
        if !self.configurations.is_empty() {
            self.status.print_sym();
            cli_print!(" Formation {}: ", self.name);
            self.status.print();
            cli_println!("\n│");

            for (i, cfg) in self.configurations.inner.iter().enumerate() {
                cfg.print_pretty(i == self.configurations.len() - 1)
            }
        } else {
            // If we have no configurations to display we assume the Formation is down
            // We have to make a new status though because we're behind a & reference. Luckily,
            // we're making an empty status struct so it's cheap.
            let mut fs = FormationStatus::new(&self.name);
            fs.status = OpStatus::Down;
            fs.status.print_sym();
            cli_print!(" Formation {}: ", fs.name);
            fs.status.print();
            cli_println!("");
        }

        Ok(())
    }
}

impl Output for Vec<FormationStatus> {
    fn print_json(&self, _ctx: &Ctx) -> Result<()> {
        cli_println!("{}", serde_json::to_string(self)?);

        Ok(())
    }

    fn print_table(&self, ctx: &Ctx) -> Result<()> {
        for fstatus in self.iter() {
            fstatus.print_table(ctx)?;
        }

        Ok(())
    }
}
