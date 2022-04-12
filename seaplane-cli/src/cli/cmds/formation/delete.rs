use clap::{ArgMatches, Command};

use crate::{
    cli::{
        cmds::{flight::SeaplaneFlightDelete, formation::build_request},
        errors,
        validator::{validate_formation_name, validate_name_id},
        CliCommand,
    },
    context::Ctx,
    error::{CliErrorKind, Context, Result},
    printer::Color,
};

#[derive(Copy, Clone, Debug)]
pub struct SeaplaneFormationDelete;

impl SeaplaneFormationDelete {
    pub fn command() -> Command<'static> {
        let validator = |s: &str| validate_name_id(validate_formation_name, s);
        // TODO: add --recursive to handle configurations too
        Command::new("delete")
            .visible_aliases(&["del", "remove", "rm"])
            .about("Deletes local Formation Plans and/or remote Formation Instances")
            .override_usage(
                "seaplane formation delete <NAME|ID> [OPTIONS]
    seaplane formation delete <NAME|ID> --no-remote [OPTIONS]")
            .arg(arg!(formation =["NAME|ID"] required)
                .validator(validator)
                .help("The name or ID of the Formation to remove, must be unambiguous"))
            .arg(arg!(--recursive -('r'))
                .help("Recursively delete all local definitions associated with this Formation"))
            .arg(arg!(--force -('f'))
                .help("Delete this Formation even if there are remote instances In Flight (active), which will effectively stop all remote instances of this Formation"))
            .arg(arg!(--all -('a'))
                .help("Delete all matching Formations even when the name or ID is ambiguous or a partial match"))
            .arg(arg!(--local)
                .overrides_with("no-local")
                .help("Delete local Formation Definitions (this is set by the default, use --no-local to skip)"))
            .arg(arg!(--("no-local"))
                .overrides_with("local")
                .help("DO NOT delete local Formation Definitions"))
            .arg(arg!(--remote)
                .overrides_with("no-remote")
                .help("Delete remote Formation Instances (this is set by default, use --no-remote to skip)"))
            .arg(arg!(--("no-remote"))
                .overrides_with("remote")
                .help("DO NOT delete remote Formation Instances (this is set by the default, use --remote to remove them)"))
    }
}

impl CliCommand for SeaplaneFormationDelete {
    fn run(&self, ctx: &mut Ctx) -> Result<()> {
        let formation_ctx = ctx.formation_ctx.get_or_init();

        if !formation_ctx.local && !formation_ctx.remote {
            cli_eprint!(@Red, "error: ");
            cli_eprintln!("nothing to do");
            cli_eprint!("(hint: either remove ");
            cli_eprint!(@Yellow, "--no-local ");
            cli_eprint!("or add ");
            cli_eprint!(@Yellow, "--remote ");
            cli_eprintln!("to the command)");
            std::process::exit(1);
        }

        // Get the indices of any formations that match the given name/ID
        let indices = if ctx.args.all {
            ctx.db
                .formations
                .formation_indices_of_left_matches(&formation_ctx.name_id)
        } else {
            ctx.db
                .formations
                .formation_indices_of_matches(&formation_ctx.name_id)
        };

        match indices.len() {
            0 => errors::no_matching_item(formation_ctx.name_id.clone(), false, ctx.args.all)?,
            1 => (),
            _ => {
                // TODO: and --force
                if !ctx.args.all {
                    errors::ambiguous_item(formation_ctx.name_id.clone(), true)?;
                }
            }
        }

        let mut deleted = indices.len();

        // Remove the Formations
        //
        // First try to delete the remote formation if required, because we don't want to delete
        // the local one too if this fails
        if formation_ctx.remote {
            let api_key = ctx.args.api_key()?;
            for idx in &indices {
                let formation = ctx.db.formations.get_formation(*idx).unwrap();
                if let Some(name) = &formation.name {
                    let delete_req = build_request(Some(name), api_key)?;
                    let cfg_uuids = delete_req.delete(ctx.args.force)?;
                    cli_print!("Deleted remote Formation Instance '");
                    cli_print!(@Green, "{}", name);
                    if cfg_uuids.is_empty() {
                        cli_println!("'");
                    } else {
                        cli_println!("' with Configuration UUIDs:");
                        for uuid in cfg_uuids.into_iter() {
                            cli_println!(@Green, "\t{uuid}");
                        }
                    }
                } else {
                    return Err(CliErrorKind::NoMatchingItem(formation_ctx.name_id.clone())
                        .into_err()
                        .context("(hint: create the local Formation Plan with '")
                        .color_context(Color::Green, "seaplane formation plan")
                        .context("')\n")
                        .context("(hint: or try synchronizing local Plan definitions with remote instances by using '")
                        .color_context(Color::Green, "seaplane formation fetch-remote")
                        .context("')\n")
                        .context("(hint: You can also fetch remote instances by adding 'seaplane formation delete ")
                        .color_context(Color::Green, "--fetch")
                        .context("')\n"));
                }
            }
        }
        if formation_ctx.local {
            // No need to potentially clone over and over
            let mut cloned_ctx = ctx.clone();
            for formation in ctx.db.formations.remove_formation_indices(&indices).iter() {
                let ids = if ctx.args.force {
                    formation.local.iter().cloned().collect()
                } else {
                    formation.local_only_configs()
                };
                for id in ids {
                    if let Some(cfg) = ctx.db.formations.remove_configuration(&id) {
                        if formation_ctx.recursive {
                            cloned_ctx.internal_run = true;
                            for flight in cfg.model.flights() {
                                let flight_name = flight.name();
                                if !ctx
                                    .db
                                    .formations
                                    .configurations()
                                    .filter(|cfg| cfg.id != id)
                                    .any(|cfg| {
                                        cfg.model.flights().iter().any(|f| f.name() == flight_name)
                                    })
                                {
                                    cloned_ctx.args.name_id = Some(flight_name.to_string());
                                    SeaplaneFlightDelete.run(&mut cloned_ctx)?;
                                    deleted += 1;
                                }
                            }
                        }
                    }
                }
                cli_println!("Deleted local Formation Plan {}", &formation.id.to_string());
            }
        }

        ctx.persist_formations()?;

        // TODO: recalculate dichotomy of local v. remote numbers (i.e. --no-local, etc.)
        cli_println!(
            "\nSuccessfully removed {} item{}",
            deleted,
            if deleted > 1 { "s" } else { "" }
        );

        Ok(())
    }

    fn update_ctx(&self, matches: &ArgMatches, ctx: &mut Ctx) -> Result<()> {
        ctx.args.force = matches.is_present("force");
        let mut fctx = ctx.formation_ctx.get_mut_or_init();
        fctx.name_id = matches.value_of("formation").unwrap().to_string();
        fctx.remote = !matches.is_present("no-remote");
        fctx.local = !matches.is_present("no-local");
        fctx.recursive = matches.is_present("recursive");

        Ok(())
    }
}
