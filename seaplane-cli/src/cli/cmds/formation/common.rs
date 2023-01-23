//! The common args effectively represent a "Formation Configuration" which many commands can
//! include.
//!
//! The only additional information is the formation name, which is not part of the configuration,
//! but many commands need as well.

use clap::{builder::PossibleValue, value_parser, Arg};

use crate::cli::validator::{
    validate_endpoint, validate_formation_name, validate_name_id, validate_name_id_path_inline,
    validate_public_endpoint,
};

static LONG_NAME: &str =
    "A human readable name for the Formation (must be unique within the tenant)

Rules for a valid name are as follows:

  - may only include 0-9, a-z, A-Z, and '-' (hyphen)
  - hyphens ('-') may not be repeated (i.e. '--')
  - no more than three (3) total hyphens
  - the total length must be <= 27

Some of these restrictions may be lifted in the future.";

static LONG_FLIGHT: &str =
    "A Flight Plan to include in this Formation in the form of ID|NAME|@path|@-|INLINE-SPEC (See FLIGHT SPEC below)

Multiple items can be passed as a SEMICOLON (';') separated list or by using the argument multiple
times. Note that when using the INLINE-SPEC it's usually easiest to only place one Flight Plan per
--include-flight-plan argument

$ seaplane formation plan \\
    --include-flight-plan name=flight1,image=nginx:latest \\
    --include-flight-plan name=flight2,image=hello:latest

Which would create, and include, two Flight Plans (flight1, and flight2).";

pub fn args() -> Vec<Arg> {
    let validator = |s: &str| validate_name_id(validate_formation_name, s);
    #[cfg_attr(not(feature = "unstable"), allow(unused_mut))]
    let mut hide = true;
    let _ = hide;
    #[cfg(feature = "unstable")]
    {
        hide = false;
    }

    // TODO: add --from with support for @file and @- (stdin)
    vec![
        arg!(name_id --name -('n') =["STRING"])
            .help("A human readable name for the Formation (must be unique within the tenant) if omitted a pseudo random name will be assigned")
            .long_help(LONG_NAME)
            .value_parser(validate_formation_name),
        arg!(--launch)
            .help("This Formation Plan should be deployed and set as active right away (requires a formation configuration)"),
        arg!(--("include-flight-plan")|("include-flight-plans") -('I') =["SPEC"]...)
            .help("Use local Flight Plan in this Formation in the form of ID|NAME|@path|@-|INLINE-SPEC (supports SEMICOLON (';') separated list, or multiple uses) (See FLIGHT SPEC below)")
            .value_delimiter(';')
            .required(true)
            .long_help(LONG_FLIGHT)
            .value_parser(validate_name_id_path_inline),
    ]
}
