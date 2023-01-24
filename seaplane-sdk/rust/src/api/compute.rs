mod error;

#[cfg(feature = "compute_api_v1")]
pub mod v1;

#[cfg(all(feature = "compute_api_v2", feature = "unstable"))]
pub mod v2;

/// The base URL for our Compute API endpoints
///
/// The compute API handles all things compute such as building `FormationConfiguration`s to
/// `Flight`s to the underlying Containers.
pub static COMPUTE_API_URL: &str = "https://compute.cplane.cloud/";
