use clap::{ArgMatches, Command};
use seaplane::api::metadata::v1::Value;

use crate::{
    api::MetadataReq,
    cli::{cmds::metadata::common, CliCommand},
    context::{Ctx, MetadataCtx},
    error::Result,
    printer::{Output, OutputFormat},
};

/// A newtype wrapper to enforce where the ArgMatches came from which reduces errors in checking if
/// values of arguments were used or not. i.e. `seaplane formation create` may not have the same
/// arguments as `seaplane account token` even though both produce an `ArgMatches`.
#[allow(missing_debug_implementations)]
pub struct SeaplaneMetadataSetArgMatches<'a>(pub &'a ArgMatches);

#[derive(Copy, Clone, Debug)]
pub struct SeaplaneMetadataSet;

impl SeaplaneMetadataSet {
    pub fn command() -> Command {
        Command::new("set")
            .visible_alias("put")
            .about("Set a metadata key-value pair")
            .arg(common::base64())
            .arg(arg!(key =["KEY"] required ).help("The key to set"))
            .arg(arg!(value =["VALUE"] required ).help("The value (@path will load the value from a path and @- will load the value from STDIN)"))
    }
}

impl CliCommand for SeaplaneMetadataSet {
    fn run(&self, ctx: &mut Ctx) -> Result<()> {
        let mut req = MetadataReq::new(ctx)?;
        let mdctx = ctx.md_ctx.get_mut_or_init();
        for kv in mdctx.kvs.iter_mut() {
            let key = kv.key.to_string();
            let value = kv.value.to_string();
            req.set_key(&key)?;
            req.put_value(Value::from_encoded(value.clone()))?;
            if ctx.args.out_format == OutputFormat::Table {
                cli_println!("Success");
            }
        }

        if ctx.args.out_format == OutputFormat::Json {
            let kvs = ctx.md_ctx.get_or_init().kvs.clone();
            kvs.print_json(ctx)?;
        }

        Ok(())
    }

    fn update_ctx(&self, matches: &ArgMatches, ctx: &mut Ctx) -> Result<()> {
        // NOTE: MetadataCtx::from_md_set is impure and tries to read from STDIN if the value is
        // `@-`
        ctx.md_ctx
            .init(MetadataCtx::from_md_set(&SeaplaneMetadataSetArgMatches(matches))?);
        ctx.args.out_format = matches.get_one("format").copied().unwrap_or_default();
        Ok(())
    }
}
