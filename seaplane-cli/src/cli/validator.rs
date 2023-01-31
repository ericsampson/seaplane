//! Utility functions used to validate various argument types on the CLI. The arguments and return
//! type are what the `clap::Arg::validator` expects.

use std::{path::Path, result::Result as StdResult};

use crate::{
    context::{FlightCtx, DEFAULT_IMAGE_REGISTRY_URL},
    error::CliErrorKind,
    ops::formation::Endpoint,
};

pub fn validate_u64(s: &str) -> StdResult<u64, String> {
    match s.parse::<u64>() {
        Err(_) => Err("value must be a valid 64-bit integer".into()),
        Ok(n) => Ok(n),
    }
}

/// Ensures a valid Endpoint
pub fn validate_endpoint(s: &str) -> StdResult<Endpoint, String> {
    match s.parse::<Endpoint>() {
        Err(e) => Err(format!("invalid endpoint SPEC: {e}")),
        Ok(ep) => Ok(ep),
    }
}

/// Ensures a valid Public Endpoint, we must special case Public Endpoints because it only supports
/// the 'http' and 'https' protocol field.
pub fn validate_public_endpoint(s: &str) -> StdResult<Endpoint, String> {
    match s.parse::<Endpoint>() {
        Ok(ep) => Ok(ep),
        Err(mut details) => {
            if details.starts_with("invalid protocol") {
                details = "invalid protocol (valid options: http, https)".into();
            }
            Err(format!("invalid endpoint SPEC: {details}"))
        }
    }
}

/// The arg can be any of:
///
/// - name
/// - Local ID (32byte hex encoded string)
/// - @- (means STDIN)
/// - @path
pub fn validate_name_id_path<F>(name_validator: F, s: &str) -> StdResult<String, String>
where
    F: Fn(&str) -> StdResult<String, &'static str>,
{
    name_validator(s)
        .or_else(|_| validate_id(s))
        .or_else(|_| validate_at_path(s))
        .or_else(|_| validate_at_stdin(s))
        .map_err(|_| {
            "the value must be a NAME|ID|@PATH|@- where @PATH is a valid path or @- opens STDIN"
                .to_owned()
        })
}

/// The arg can be any of:
///
/// - name
/// - Local ID (32byte hex encoded string)
/// - @- (means STDIN)
/// - @path
/// - INLINE-SPEC for Flights
pub fn validate_name_id_path_inline(s: &str) -> StdResult<String, String> {
    validate_flight_name(s)
        .or_else(|_| validate_id(s))
        .or_else(|_| validate_at_path(s))
        .or_else(|_| validate_at_stdin(s))
        .or_else(|_| validate_inline_flight_spec(s))
        .map_err(|s| {
            format!("the value must be a NAME|ID|@PATH|@-|INLINE-SPEC where @PATH is a valid path or @- opens STDIN\n\ncaused by: {s}")
        })
}

/// The arg can be any of:
///
/// - name
/// - Local ID (32byte hex encoded string)
pub fn validate_name_id<F>(name_validator: F, s: &str) -> StdResult<String, &'static str>
where
    F: Fn(&str) -> StdResult<String, &'static str>,
{
    name_validator(s).or_else(|_| validate_id(s))
}

/// Rules:
///
/// - 1-63 alphanumeric characters (only ASCII lowercase) and hyphens ( 0-9, a-z, A-Z, and '-' )
/// (aka, one DNS segment)
pub fn validate_flight_name(name: &str) -> StdResult<String, &'static str> {
    if name.is_empty() {
        return Err("Flight name cannot be empty");
    }
    if name.len() > 63 {
        return Err("Flight name too long, must be <= 63 in length");
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return Err(
            "illegal character in Flight name; must only contain ASCII lowercase, digit, or hyphen ('-')",
        );
    }
    if name.contains("--") {
        return Err("repeated hyphens ('--') not allowed in Flight name");
    }

    Ok(name.into())
}

/// Validate formation name using the API rules
pub fn validate_formation_name(name: &str) -> StdResult<String, String> {
    seaplane::api::compute::v2::validate_formation_name(name).map_err(|e| e.to_string())?;
    Ok(name.into())
}

/// The value may be `@path` where `path` is some path that exists
pub fn validate_at_path(s: &str) -> StdResult<String, String> {
    if let Some(path) = s.strip_prefix('@') {
        if !Path::exists(path.as_ref()) {
            #[cfg(not(any(feature = "semantic_ui_tests", feature = "ui_tests")))]
            return Err(format!("path '{path}' does not exist"));
        }
    } else {
        return Err("the '@<path>'  was not provided".to_owned());
    }

    Ok(s.into())
}

/// The value may be `@-`
pub fn validate_at_stdin(s: &str) -> StdResult<String, &'static str> {
    if s != "@-" {
        return Err("the value '@-' was not provided");
    }

    Ok(s.into())
}

/// The value may be:
///
/// name=NAME,image=IMG,maximum=NUM,minimum=NUM,api-permission[=true|false],architecture=ARCH
///
/// where only image is required, and architecture can be passed multiple times.
pub fn validate_inline_flight_spec(s: &str) -> StdResult<String, String> {
    // We use the default image registry URL regardless of what the user has set because we're only
    // checking validity, not actually using this data
    if let Err(e) = FlightCtx::from_inline_flight(s, DEFAULT_IMAGE_REGISTRY_URL) {
        return Err(format!("invalid INLINE-SPEC: {}", {
            match e.kind() {
                CliErrorKind::InlineFlightUnknownItem(s) => format!("unknown item {s}"),
                CliErrorKind::InlineFlightMissingValue(s) => {
                    format!("key '{s}' is missing the value")
                }
                CliErrorKind::InlineFlightHasSpace => {
                    String::from("inline flight contains a space (' ')")
                }
                CliErrorKind::InlineFlightMissingImage => {
                    String::from("Missing required image=IMAGE-SPEC")
                }
                CliErrorKind::InlineFlightInvalidName(s) => {
                    format!("Flight Plan name '{s}' isn't valid")
                }
                CliErrorKind::ParseInt(e) => format!("failed to parse minimum or maximum: {e}"),
                CliErrorKind::StrumParse(e) => format!("failed to parse architecture: {e}"),
                CliErrorKind::ImageReference(e) => format!("invalid IMAGE-SPEC: {e}"),
                _ => unreachable!(),
            }
        }));
    }
    Ok(s.into())
}

/// The value may be a *up to* a 32byte hex encoded string
pub fn validate_id(s: &str) -> StdResult<String, &'static str> {
    if s.is_empty() {
        return Err("ID cannot be empty");
    }
    if !s.chars().all(is_hex_char) {
        return Err("found non-hex character");
    }
    if s.chars().count() > 64 {
        return Err("ID provided is too long");
    }

    Ok(s.into())
}

fn is_hex_char(c: char) -> bool { matches!(c, 'a'..='f' | 'A'..='F' | '0'..='9') }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_hex_char_valid() {
        for c in ('a'..'g').chain(('A'..'G').chain('0'..='9')) {
            assert!(is_hex_char(c));
        }
    }

    #[test]
    fn is_hex_char_invalid() {
        for c in ('g'..='z').chain('G'..'Z') {
            assert!(!is_hex_char(c));
        }
    }

    #[test]
    fn invalid_flight_names() {
        assert!(validate_flight_name("").is_err());
        assert!(validate_flight_name("no-special-chars!").is_err());
        assert!(validate_flight_name("imwaaaaaytoolongforanythingthatshouldbeanameimwaaaaaytoolongforanythingthatshouldbeaname").is_err());
        assert!(validate_flight_name("noUperCase").is_err());
    }

    #[test]
    fn invalid_formation_names() {
        assert!(validate_formation_name("").is_err());
        assert!(validate_formation_name("no-special-chars!").is_err());
        assert!(validate_formation_name("imwaaaaaytoolongforanythingthatshouldbeaname").is_err());
        assert!(validate_formation_name("too-many-hyphens-in-here").is_err());
        assert!(validate_formation_name("no-ending-hyphen-").is_err());
        assert!(validate_formation_name("noUperCase").is_err());
    }

    #[test]
    fn invalid_id() {
        assert!(validate_id("").is_err());
        assert!(validate_id("imnotahexvalue").is_err());
        // Too long
        assert!(validate_id("abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890").is_err());
    }

    #[test]
    fn invalid_at_path() {
        assert!(validate_id("").is_err());
        assert!(validate_id("@").is_err());
        assert!(validate_id("@-").is_err());
        assert!(validate_id("@foo").is_err());
        assert!(validate_id("foo").is_err());
    }

    #[test]
    fn invalid_at_stdin() {
        assert!(validate_id("").is_err());
        assert!(validate_id("@").is_err());
        assert!(validate_id("@foo").is_err());
        assert!(validate_id("foo").is_err());
    }
}
