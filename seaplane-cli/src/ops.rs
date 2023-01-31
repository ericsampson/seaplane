//! This module provides types that wrap the API endpoint models and add additional fields/context
//! that is only relevant for the CLI or purposes of consuming the API.

pub mod db;
pub mod encoded_string;
pub mod formation;
pub mod locks;
pub mod metadata;
pub mod restrict;
pub mod state_version;

use std::fmt;

use rand::Rng;
use serde::{Deserialize, Serialize};

pub use self::encoded_string::EncodedString;
use crate::cli::validator::{validate_flight_name, validate_formation_name};

pub fn generate_flight_name() -> String {
    // TODO: Maybe set an upper bound on the number of iterations and don't expect
    names::Generator::default()
        .find(|name| validate_flight_name(name).is_ok())
        .expect("Failed to generate a random name")
}

pub fn generate_formation_name() -> String {
    // TODO: Maybe set an upper bound on the number of iterations and don't expect
    names::Generator::default()
        .find(|name| validate_formation_name(name).is_ok())
        .expect("Failed to generate a random name")
}
