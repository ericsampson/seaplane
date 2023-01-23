//! Context describes the normalized and processed "settings" that a command can use at runtime.
//! This differs from the "config" or the "CLI Arguments" as the Context is built and updated from
//! those sources. This means the context is responsible for de-conflicting mutually exclusive
//! options, or overriding values.
//!
//! The Context is source of truth for all runtime decisions.
//!
//! The order of evaluation is as follows (note lower layers override layers above):
//!
//! 1. System configuration files are loaded (if any...currently none are defined)
//! 2. User configuration files are loaded (if any are found)
//! 3. Environment Variables (currently none are defined)
//! 4. Command Line Arguments
//!   4a. Because we use subcommands and global arguments each subcommand acts as it's own set of
//!   Command Line Arguments, and can thus affect the Context at each level in the command
//! hierarchy.   4b. Before the Context is handed off mutably to the next nested level, all updates
//! from the   parent should be finalized.
//!
//! After these steps the final Context is what is used to make runtime decisions.
//!
//! The context struct itself contains "global" values or those that apply in many scenarios or to
//! many commands. It also contains specialized contexts that contain values only relevant to those
//! commands or processes that need them. These specialized contexts should be lazily derived.

pub mod formation;
pub use formation::{FormationCfgCtx, FormationCtx};
pub mod metadata;
pub use metadata::MetadataCtx;
pub mod locks;
pub use locks::LocksCtx;
pub mod restrict;
use std::path::{Path, PathBuf};

use clap_complete::Shell;
use once_cell::unsync::OnceCell;
use reqwest::Url;
pub use restrict::RestrictCtx;

use crate::{
    config::RawConfig,
    error::{CliErrorKind, Context, Result},
<<<<<<< HEAD
    fs::{self, ToDisk},
    ops::db::Db,
    printer::{ColorChoice, OutputFormat},
};

// Used in state v0
const FLIGHTS_FILE: &str = "flights.json";
=======
    fs::{self, FromDisk, ToDisk},
    ops::formation::Formations,
    printer::{ColorChoice, OutputFormat},
};

>>>>>>> 23cf35f (remove flights as objects)
const FORMATIONS_FILE: &str = "formations.json";
// Used in state v1
const STATE_FILE: &str = "state.json";
/// The registry to use for image references when the registry is omitted by the user
pub const DEFAULT_IMAGE_REGISTRY_URL: &str = "registry.cplane.cloud";

#[derive(Debug, Default, Clone)]
pub struct Args {
    /// when the answer is "yes...but only if the stream is a TTY." In these cases an enum of
    /// Never, Auto, Always would be more appropriate
    ///
    /// Should be display ANSI color codes in output?
    pub color: ColorChoice,

    /// The name or local ID of an item
    pub name_id: Option<String>,

    /// What to overwrite
    pub overwrite: Vec<String>,

    /// Do items need to be exact to match
    pub exact: bool,

    /// Match all items
    pub all: bool,

    /// Display third party licenses
    pub third_party: bool,

    /// The shell to generate completions for
    pub shell: Option<Shell>,

    /// How to display output
    pub out_format: OutputFormat,

    /// Try to force the operation to happen
    pub force: bool,

    /// Do not use local state files
    pub stateless: bool,

    /// The API Key associated with an account provided by the CLI, env, or Config used to request
    /// access tokens
    pub api_key: Option<String>,

    /// Should we fetch remote refs?
    pub fetch: bool,
}

impl Args {
    pub fn api_key(&self) -> Result<&str> {
        self.api_key
            .as_deref()
            .ok_or_else(|| CliErrorKind::MissingApiKey.into_err())
    }
}

/// The source of truth "Context" that is passed to all runtime processes to make decisions based
/// on user configuration
// TODO: we may not want to derive this we implement circular references
#[derive(Debug)]
pub struct Ctx {
    /// The platform specific path to a data location
    data_dir: PathBuf,

    /// Context relate to exclusively to Formation operations and commands
    pub formation_ctx: LateInit<FormationCtx>,

    /// Context relate to exclusively to key-value operations and commands
    pub md_ctx: LateInit<MetadataCtx>,

    /// Context relate to exclusively to Locks operations and commands
    pub locks_ctx: LateInit<LocksCtx>,

    /// Context relate to exclusively to Restrict operations and commands
    pub restrict_ctx: LateInit<RestrictCtx>,

    /// Where the configuration files were loaded from
    pub conf_files: Vec<PathBuf>,

    /// Common CLI arguments
    pub args: Args,

    /// The in memory databases
    pub db: Db,

    /// Allows tracking if we're running a command internally and skippy certain checks or output
    pub internal_run: bool,

    /// Did we run initialization automatically or not on startup?
    pub did_init: bool,

    /// Disable progress bar indicators
    pub disable_pb: bool,

    /// The container image registry to infer if not provided
    pub registry: String,

    /// Set the base URL for the request
    pub compute_url: Option<Url>,
    pub identity_url: Option<Url>,
    pub metadata_url: Option<Url>,
    pub locks_url: Option<Url>,
    pub insecure_urls: bool,
    pub invalid_certs: bool,
}

impl Clone for Ctx {
    fn clone(&self) -> Self {
        Self {
            data_dir: self.data_dir.clone(),
            formation_ctx: if self.formation_ctx.get().is_some() {
                let li = LateInit::default();
                li.init(self.formation_ctx.get().cloned().unwrap());
                li
            } else {
                LateInit::default()
            },
            md_ctx: if self.md_ctx.get().is_some() {
                let li = LateInit::default();
                li.init(self.md_ctx.get().cloned().unwrap());
                li
            } else {
                LateInit::default()
            },
            locks_ctx: if self.locks_ctx.get().is_some() {
                let li = LateInit::default();
                li.init(self.locks_ctx.get().cloned().unwrap());
                li
            } else {
                LateInit::default()
            },
            restrict_ctx: if self.restrict_ctx.get().is_some() {
                let li = LateInit::default();
                li.init(self.restrict_ctx.get().cloned().unwrap());
                li
            } else {
                LateInit::default()
            },
            conf_files: self.conf_files.clone(),
            args: self.args.clone(),
            db: self.db.clone(),
            internal_run: self.internal_run,
            did_init: self.did_init,
            disable_pb: self.disable_pb,
            registry: self.registry.clone(),
            compute_url: self.compute_url.clone(),
            identity_url: self.identity_url.clone(),
            metadata_url: self.metadata_url.clone(),
            locks_url: self.locks_url.clone(),
            insecure_urls: self.insecure_urls,
            invalid_certs: self.invalid_certs,
        }
    }
}

impl Default for Ctx {
    fn default() -> Self {
        Self {
            data_dir: fs::data_dir(),
            formation_ctx: LateInit::default(),
            md_ctx: LateInit::default(),
            locks_ctx: LateInit::default(),
            restrict_ctx: LateInit::default(),
            conf_files: Vec::new(),
            args: Args::default(),
            db: Db::default(),
            internal_run: false,
            did_init: false,
            disable_pb: false,
            compute_url: None,
            identity_url: None,
            metadata_url: None,
            locks_url: None,
            insecure_urls: false,
            invalid_certs: false,
            registry: DEFAULT_IMAGE_REGISTRY_URL.into(),
        }
    }
}

impl From<RawConfig> for Ctx {
    fn from(cfg: RawConfig) -> Self {
        Self {
            data_dir: fs::data_dir(),
            conf_files: cfg.loaded_from.clone(),
            args: Args {
                // We default to using color. Later when the context is updated from the CLI args,
                // this may change.
                color: cfg.seaplane.color.unwrap_or_default(),
                api_key: cfg.account.api_key,
                ..Default::default()
            },
            registry: cfg
                .seaplane
                .default_registry_url
                .unwrap_or_else(|| DEFAULT_IMAGE_REGISTRY_URL.into())
                .trim_end_matches('/')
                .to_string(),
            compute_url: cfg.api.compute_url,
            identity_url: cfg.api.identity_url,
            metadata_url: cfg.api.metadata_url,
            locks_url: cfg.api.locks_url,
            did_init: cfg.did_init,
            #[cfg(feature = "allow_insecure_urls")]
            insecure_urls: cfg.danger_zone.allow_insecure_urls,
            #[cfg(feature = "allow_invalid_certs")]
            invalid_certs: cfg.danger_zone.allow_invalid_certs,
            ..Self::default()
        }
    }
}

impl Ctx {
    pub fn update_from_env(&mut self) -> Result<()> {
        // TODO: this just gets it compiling. Using `todo!` blocks progress since loading the
        // context happens at program startup, so we cannot panic on unimplemented
        Ok(())
    }

    #[inline]
    pub fn data_dir(&self) -> &Path { &self.data_dir }

    pub fn conf_files(&self) -> &[PathBuf] { &self.conf_files }

<<<<<<< HEAD
    pub fn state_file(&self) -> PathBuf { self.data_dir.join(STATE_FILE) }
    pub fn flights_file(&self) -> PathBuf { self.data_dir.join(FLIGHTS_FILE) }
    pub fn formations_file(&self) -> PathBuf { self.data_dir.join(FORMATIONS_FILE) }

    /// Write out an entirely new JSON file if `--stateless` wasn't used
    pub fn persist_state(&self) -> Result<()> {
        if !self.args.stateless {
            self.db
                .persist()
                .with_context(|| format!("Path: {:?}\n", self.state_file()))?
        }
        Ok(())
=======
    pub fn formations_file(&self) -> PathBuf { self.data_dir.join(FORMATIONS_FILE) }

    /// Write out an entirely new JSON file if `--stateless` wasn't used
    pub fn persist_formations(&self) -> Result<()> {
        self.db
            .formations
            .persist_if(!self.args.stateless)
            .with_context(|| format!("Path: {:?}\n", self.formations_file()))
    }
}

/// The in memory "Databases"
#[derive(Debug, Default, Clone)]
pub struct Db {
    /// The in memory Formations database
    pub formations: Formations,

    /// A *hint* that we should persist at some point. Not gospel
    pub needs_persist: bool,
}

impl Db {
    pub fn load<P: AsRef<Path>>(formations: P) -> Result<Self> { Self::load_if(formations, true) }

    pub fn load_if<P: AsRef<Path>>(formations: P, yes: bool) -> Result<Self> {
        Ok(Self {
            formations: FromDisk::load_if(formations, yes)
                .unwrap_or_else(|| Ok(Formations::default()))?,
            needs_persist: false,
        })
>>>>>>> 23cf35f (remove flights as objects)
    }
}

// TODO: we may not want to derive this we implement circular references
#[derive(Debug)]
pub struct LateInit<T> {
    inner: OnceCell<T>,
}

impl<T> Default for LateInit<T> {
    fn default() -> Self { Self { inner: OnceCell::default() } }
}

impl<T> LateInit<T> {
    pub fn init(&self, val: T) { assert!(self.inner.set(val).is_ok()) }
    pub fn get(&self) -> Option<&T> { self.inner.get() }
    pub fn get_mut(&mut self) -> Option<&mut T> { self.inner.get_mut() }
}

impl<T: Default> LateInit<T> {
    pub fn get_or_init(&self) -> &T { self.inner.get_or_init(|| T::default()) }
    pub fn get_mut_or_init(&mut self) -> &mut T {
        self.inner.get_or_init(|| T::default());
        self.inner.get_mut().unwrap()
    }
}
