pub mod cmds;
pub mod errors;
pub mod specs;
pub mod validator;

use std::env;
#[cfg(not(any(feature = "api_tests", feature = "semantic_ui_tests", feature = "ui_tests")))]
use std::io::{self, BufRead};

use clap::{crate_authors, value_parser, ArgAction, ArgMatches, Command};
use const_format::concatcp;

pub use crate::cli::cmds::*;
#[cfg(not(any(feature = "api_tests", feature = "semantic_ui_tests", feature = "ui_tests")))]
use crate::ops::db::Db;
use crate::{
    context::Ctx,
    error::Result,
    printer::{ColorChoice, Printer},
};

const VERSION: &str = env!("SEAPLANE_VER_WITH_HASH");
static AUTHORS: &str = crate_authors!();
static LONG_VERBOSE: &str = "Display more verbose output

More uses displays more verbose output
    -v:  Display debug info
    -vv: Display trace info";

static LONG_QUIET: &str = "Suppress output at a specific level and below

More uses suppresses higher levels of output
    -q:   Only display WARN messages and above
    -qq:  Only display ERROR messages
    -qqq: Suppress all output";
static LONG_API_KEY: &str =
    "The API key associated with a Seaplane account used to access Seaplane API endpoints

The value provided here will override any provided in any configuration files.
A CLI provided value also overrides any environment variables.
One can use a special value of '-' to signal the value should be read from STDIN.";

pub trait CliCommand {
    /// Care should be taken to keep CliCommand::update_ctx pure with no external effects such as
    /// I/O. This allows the CLI to be fully tested without any assumptions of the testing
    /// environment
    fn update_ctx(&self, _matches: &ArgMatches, _ctx: &mut Ctx) -> Result<()> { Ok(()) }
    fn run(&self, _ctx: &mut Ctx) -> Result<()> { Ok(()) }
    fn next_subcmd<'a>(
        &self,
        _matches: &'a ArgMatches,
    ) -> Option<(Box<dyn CliCommand>, &'a ArgMatches)> {
        None
    }
}

impl dyn CliCommand + 'static {
    /// Performs three steps:
    ///
    /// - calls `self.update_ctx()`
    /// - calls `self.run()`
    /// - Gets the next subcommand (if any) by calling `self.next_subcmd()` and calls
    /// `traverse_exec` on that subcommand.
    ///
    /// This walks down the entire *used* subcommand hierarchy ensuring the `update_ctx` was called
    /// prior to `run` and that any deeper subcommands were executed.
    pub fn traverse_exec(&self, matches: &ArgMatches, ctx: &mut Ctx) -> Result<()> {
        self.update_ctx(matches, ctx)?;
        self.run(ctx)?;
        if let Some((c, m)) = self.next_subcmd(matches) {
            return c.traverse_exec(m, ctx);
        }
        Ok(())
    }

    // Used testing the CLI to cause the CliCommand::update_ctx to be called, but not
    // CliCommand::run
    pub fn traverse_update_ctx(&self, matches: &ArgMatches, ctx: &mut Ctx) -> Result<()> {
        self.update_ctx(matches, ctx)?;
        if let Some((c, m)) = self.next_subcmd(matches) {
            return c.traverse_update_ctx(m, ctx);
        }
        Ok(())
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Seaplane;

impl Seaplane {
    pub fn command() -> Command {
        #[cfg_attr(not(any(feature = "unstable", feature = "ui_tests")), allow(unused_mut))]
        let mut app = Command::new("seaplane")
            .about("Seaplane CLI for managing resources on the Seaplane Cloud")
            .author(AUTHORS)
            .version(VERSION)
            .long_version(concatcp!(VERSION, "\n", env!("SEAPLANE_BUILD_FEATURES")))
            .propagate_version(true)
            .subcommand_required(true)
            .arg_required_else_help(true)
            .arg(arg!(--verbose -('v') global)
                .help("Display more verbose output")
                .action(ArgAction::Count)
                .long_help(LONG_VERBOSE))
            .arg(arg!(--quiet -('q') global)
                .help("Suppress output at a specific level and below")
                .action(ArgAction::Count)
                .long_help(LONG_QUIET))
            .arg(arg!(--color global ignore_case =["COLOR"=>"auto"])
                .value_parser(value_parser!(ColorChoice))
                .overrides_with_all(["color", "no-color"])
                .help("Should the output include color?"))
            .arg(arg!(--("no-color") global)
                .overrides_with_all(["color", "no-color"])
                .help("Do not color output (alias for --color=never)"))
            .arg(arg!(--("api-key") -('A') global =["STRING"] hide_env_values)
                .env("SEAPLANE_API_KEY")
                .help("The API key associated with a Seaplane account used to access Seaplane API endpoints")
                .long_help(LONG_API_KEY))
            .arg(arg!(--("stateless") -('S') global)
                .help("Ignore local state files, do not read from or write to them"))
            .subcommand(SeaplaneAccount::command())
            .subcommand(SeaplaneFormation::command())
            .subcommand(SeaplaneInit::command())
            .subcommand(SeaplaneLicense::command())
            .subcommand(SeaplaneMetadata::command())
            .subcommand(SeaplaneLocks::command())
            .subcommand(SeaplaneRestrict::command())
            .subcommand(SeaplaneShellCompletion::command());

        #[cfg(feature = "unstable")]
        {
            app = app
                .subcommand(SeaplaneConfig::command())
                .subcommand(SeaplaneImage::command());
        }

        #[cfg(feature = "ui_tests")]
        {
            app = app.term_width(0);
        }
        app
    }
}

impl CliCommand for Seaplane {
    fn run(&self, ctx: &mut Ctx) -> Result<()> {
        // Initialize the printer now that we have all the color choices
        Printer::init(ctx.args.color);
        Ok(())
    }

    fn update_ctx(&self, matches: &ArgMatches, ctx: &mut Ctx) -> Result<()> {
        // There is a "bug" where due to how clap handles nested-subcommands with global flags and
        // overrides (yeah...niche) if two mutually exclusive flags that override each-other are
        // used at different nesting levels, the overrides do not happen.
        //
        // For us this means doing `seaplane --no-color SUBCOMMAND --color=auto` effectively there
        // will be no color output, because clap will evaluate `--no-color` to `true` (i.e. used)
        // even though they override each-other.
        //
        // So we err on the side of not providing color since that is the safer option
        ctx.args.color = match (matches.get_one("color").copied(), matches.get_flag("no-color")) {
            (_, true) => ColorChoice::Never,
            (Some(choice), _) => {
                if choice != ColorChoice::Auto {
                    choice
                } else {
                    ctx.args.color
                }
            }
            _ => unreachable!("neither --color nor --no-color were used somehow"),
        };

        ctx.args.stateless = matches.get_flag("stateless");

        // API tests sometimes write their own DB to test, so we don't want to overwrite that
        #[cfg(not(any(
            feature = "api_tests",
            feature = "semantic_ui_tests",
            feature = "ui_tests"
        )))]
        {
<<<<<<< HEAD
            if !ctx.args.stateless {
                ctx.db = Db::load_and_upgrade(ctx)?;
            }
=======
            ctx.db = crate::context::Db::load_if(ctx.formations_file(), !ctx.args.stateless)?;
>>>>>>> 23cf35f (remove flights as objects)
        }

        if let Some(key) = &matches.get_one::<String>("api-key") {
            if key == &"-" {
                // We don't want to read from STDIN during tests
                #[cfg(not(any(
                    feature = "api_tests",
                    feature = "semantic_ui_tests",
                    feature = "ui_tests"
                )))]
                {
                    let stdin = io::stdin();
                    let mut lines = stdin.lock().lines();
                    if let Some(line) = lines.next() {
                        ctx.args.api_key = Some(line?);
                    }
                }
            } else {
                ctx.args.api_key = Some(key.to_string());
            }
        }

        Ok(())
    }

    fn next_subcmd<'a>(
        &self,
        matches: &'a ArgMatches,
    ) -> Option<(Box<dyn CliCommand>, &'a ArgMatches)> {
        match matches.subcommand() {
            Some(("account", m)) => Some((Box::new(SeaplaneAccount), m)),
            Some(("formation", m)) => Some((Box::new(SeaplaneFormation), m)),
            Some(("init", m)) => Some((Box::new(SeaplaneInit), m)),
            Some(("metadata", m)) => Some((Box::new(SeaplaneMetadata), m)),
            Some(("locks", m)) => Some((Box::new(SeaplaneLocks), m)),
            Some(("restrict", m)) => Some((Box::new(SeaplaneRestrict), m)),
            Some(("shell-completion", m)) => Some((Box::new(SeaplaneShellCompletion), m)),
            Some(("license", m)) => Some((Box::new(SeaplaneLicense), m)),
            #[cfg(feature = "unstable")]
            Some(("image", m)) => Some((Box::new(SeaplaneImage), m)),
            #[cfg(feature = "unstable")]
            Some(("config", m)) => Some((Box::new(SeaplaneConfig), m)),
            _ => None, // TODO: handle external plugins
        }
    }
}
