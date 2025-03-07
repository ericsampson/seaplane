// Copyright ⓒ  2022 Seaplane IO, Inc.
// Licensed under the Apache 2.0 license
// (see LICENSE or <http://opensource.org/licenses/Apache-2.0>) All files in the project carrying such
// notice may not be copied, modified, or distributed except according to those terms.

use seaplane_cli::{
    cli::{CliCommand, Seaplane},
    config::RawConfig,
    context::Ctx,
    error::Result,
    log::LogLevel,
};

fn try_main() -> Result<()> {
    let matches = Seaplane::command().get_matches();
    // Normally, this would be in the Seaplane::run method, however setting up logging has to
    // happen super early in the process lifetime
    match matches.get_one::<u8>("verbose").copied() {
        Some(0) => match matches.get_one::<u8>("quiet").copied() {
            Some(0) => seaplane_cli::log::LOG_LEVEL.set(LogLevel::Info).unwrap(),
            Some(1) => seaplane_cli::log::LOG_LEVEL.set(LogLevel::Warn).unwrap(),
            Some(2) => seaplane_cli::log::LOG_LEVEL.set(LogLevel::Error).unwrap(),
            _ => seaplane_cli::log::LOG_LEVEL.set(LogLevel::Off).unwrap(),
        },
        Some(1) => seaplane_cli::log::LOG_LEVEL.set(LogLevel::Debug).unwrap(),
        _ => seaplane_cli::log::LOG_LEVEL.set(LogLevel::Trace).unwrap(),
    }

    let mut ctx =
        if !matches.get_flag("stateless") { RawConfig::load_all()?.into() } else { Ctx::default() };
    ctx.update_from_env()?;

    let s: Box<dyn CliCommand> = Box::new(Seaplane);
    s.traverse_exec(&matches, &mut ctx)?;
    Ok(())
}

fn main() {
    if let Err(e) = try_main() {
        e.exit();
    }
}
