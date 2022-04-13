use clap::{ArgMatches, Command};

use crate::{
    cli::{
        cmds::formation::build_request,
        errors,
        validator::{validate_formation_name, validate_name_id},
        CliCommand,
    },
    error::Result,
    Ctx,
};

#[derive(Copy, Clone, Debug)]
pub struct SeaplaneFormationLand;

impl SeaplaneFormationLand {
    pub fn command() -> Command<'static> {
        let validator = |s: &str| validate_name_id(validate_formation_name, s);
        Command::new("land")
            .visible_alias("stop")
            .about("Land (Stop) all configurations of a remote Formation Instance")
            .arg(
                arg!(name_id =["NAME|ID"] required)
                    .help("The name or ID of the Formation Instance to land")
                    .validator(validator),
            )
            .arg(
                arg!(--all - ('a'))
                    .help("Stop all matching Formations even when FORMATION is ambiguous"),
            )
    }
}

impl CliCommand for SeaplaneFormationLand {
    fn run(&self, ctx: &mut Ctx) -> Result<()> {
        let name = ctx.args.name_id.as_ref().unwrap();
        // Get the indices of any formations that match the given name/ID
        let indices = if ctx.args.all {
            ctx.db.formations.formation_indices_of_left_matches(name)
        } else {
            ctx.db.formations.formation_indices_of_matches(name)
        };

        match indices.len() {
            0 => errors::no_matching_item(name.to_string(), false, ctx.args.all)?,
            1 => (),
            _ => {
                // TODO: and --force
                if !ctx.args.all {
                    errors::ambiguous_item(name.to_string(), true)?;
                }
            }
        }

        let api_key = ctx.args.api_key()?;
        for idx in indices {
            // re unwrap: the indices returned came from Formations so they have to be valid
            let formation = ctx.db.formations.get_formation_mut(idx).unwrap();

            // re unwrap: We got the formation from the local DB so it has to have a name
            let stop_req = build_request(Some(formation.name.as_ref().unwrap()), api_key)?;
            stop_req.stop()?;

            // Move all configurations from in air to grounded
            let ids: Vec<_> = formation.in_air.drain().collect();
            for id in ids {
                formation.grounded.insert(id);
            }

            ctx.persist_formations()?;

            cli_print!("Successfully Landed remote Formation Instance '");
            cli_print!(@Green, "{}", &name);
            cli_println!("'");
        }

        Ok(())
    }

    fn update_ctx(&self, matches: &ArgMatches, ctx: &mut Ctx) -> Result<()> {
        ctx.args.all = matches.is_present("all");
        ctx.args.name_id = matches.value_of("name_id").map(ToOwned::to_owned);
        Ok(())
    }
}
