use std::fs;

use clap::{ArgMatches, Command};
use serde_json::json;

use crate::{
    cli::CliCommand, config::RawConfig, context::Ctx, error::Result, fs::conf_dirs,
    ops::state_version::CURRENT_STATE_VERSION,
};

static LONG_FORCE: &str =
    "Force create the files and directories (DANGER: will overwrite existing files)

Using --force is the same as using --overwrite=all";
static LONG_OVERWRITE: &str =
    "Overwrite select files or directories (DANGER: will overwrite existing data)

Using --overwrite=all is the same as using --force

Multiple items can be passed as a comma separated list, or by using the argument
multiple times.";

#[derive(Copy, Clone, Debug)]
pub struct SeaplaneInit;

impl SeaplaneInit {
    pub fn command() -> Command {
        Command::new("init")
            .about("Create the Seaplane directory structure at the appropriate locations")
            .arg(arg!(--force)
                .help("Force create the files and directories (DANGER: will overwrite existing files)")
                .long_help(LONG_FORCE))
            .arg(arg!(--overwrite =["ITEM"]...)
                .help("Overwrite select files or directories (DANGER: will overwrite existing data) (supports comma separated list, or multiple uses)")
                .long_help(LONG_OVERWRITE)
                .value_parser(["all", "formations", "config"]))
    }
}

impl CliCommand for SeaplaneInit {
    fn run(&self, ctx: &mut Ctx) -> Result<()> {
        // Create the data directory
        cli_debugln!("Creating or using data directory {:?}", ctx.data_dir());
        fs::create_dir_all(ctx.data_dir())?;

        // We only create the first (most preferred) configuration dir. If the user creates more
        // down our search path, that's fine, but we only create and advertise the first.
        let conf_dir = &conf_dirs()[0];
        cli_debugln!("Creating or using config directory {conf_dir:?}");
        fs::create_dir_all(conf_dir)?;

        // Tuple below is: (File, "empty" bytes, it's --force=OPTION)
        let to_create = &[
            (
                conf_dir.join("seaplane.toml"),
                toml::to_string_pretty(&RawConfig::default()).unwrap(),
                "config",
            ),
<<<<<<< HEAD
            (
                ctx.state_file(),
                json!({ "state_version": CURRENT_STATE_VERSION }).to_string(),
                "formations",
            ),
=======
            (ctx.formations_file(), "{}".to_string(), "formations"),
>>>>>>> 23cf35f (remove flights as objects)
        ];
        // TODO: @security create the file with limited permissions
        let mut did_create = false;
        for (file, empty_bytes, opt) in to_create {
            if file.exists() && !(ctx.did_init || ctx.internal_run) {
                // Due to how match guards work, we can't use them, we have to use if-else
                if ctx.args.force
                    || ctx
                        .args
                        .overwrite
                        .iter()
                        .any(|item| item == opt || item == "all")
                {
                    cli_debugln!(
                        "overwriting existing file {file:?} due to {}",
                        if ctx.args.force {
                            "--force".into()
                        } else {
                            format!(
                                "--overwrite={}",
                                if ctx.args.overwrite.iter().any(|item| item == "all") {
                                    "all"
                                } else {
                                    opt
                                }
                            )
                        }
                    );
                } else if ctx.args.overwrite.is_empty() {
                    // We only want to advertise the *least* destructive option, not --force or
                    // --overwrite=all. The user can find those on their own.
                    cli_debugln!("found existing file {file:?}");
                    cli_debugln!(
                        "(hint: use 'seaplane init --overwrite={opt}' to erase and overwrite it)"
                    );
                    continue;
                }
            }
            did_create = true;
            cli_debugln!("creating file {file:?}");
            fs::write(file, empty_bytes)?;
        }

        if !ctx.internal_run {
            if did_create {
                cli_println!("Successfully created Seaplane files and directories");
            } else {
                cli_println!("All Seaplane files and directories already exist");
            }
        }

        Ok(())
    }

    fn update_ctx(&self, matches: &ArgMatches, ctx: &mut Ctx) -> Result<()> {
        ctx.args.force = matches.get_flag("force");
        ctx.args.overwrite = matches
            .get_many::<String>("overwrite")
            .unwrap_or_default()
            .map(ToOwned::to_owned)
            .collect();

        Ok(())
    }
}
