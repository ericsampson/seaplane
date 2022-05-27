use serde::{Deserialize, Serialize};

use crate::{base64::Base64Encoded, impl_base64};

/// A single lock name, encoded in url-safe base64, may not contain `\0` bytes
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct LockName {
    inner: Base64Encoded,
}

impl_base64!(LockName);

/// An ID to a held lock instance
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LockID {
    inner: Base64Encoded,
}
impl_base64!(LockID);

/// A lock that at some point was held by this client.
/// At any point this may have lapsed.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct HeldLock {
    pub(crate) name: LockName,
    pub(crate) id: LockID,
    pub(crate) sequencer: u32,
}

impl HeldLock {
    /// Get a reference to the held lock's name.
    pub fn name(&self) -> &LockName {
        &self.name
    }

    /// Get a reference to the held lock's id.
    pub fn id(&self) -> &LockID {
        &self.id
    }

    /// Get the held lock's sequencer.
    pub fn sequencer(&self) -> u32 {
        self.sequencer
    }

    /// Used in testing for mapping responses
    #[cfg(feature = "api_tests")]
    pub fn new(name: LockName, id: LockID, sequencer: u32) -> HeldLock {
        HeldLock {
            name,
            id,
            sequencer,
        }
    }
}

/// The target of a request, representing a single lock.
// This will eventually have the ability to query ranges much like the metadata service
#[non_exhaustive]
#[derive(Debug)]
pub enum RequestTarget {
    SingleLock(LockName),
    HeldLock(HeldLock),
}

/// Information about an existing held lockn
#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
pub struct LockInfo {
    pub name: LockName,
    pub id: LockID,
    pub info: LockInfoInner,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
pub struct LockInfoInner {
    pub ttl: u32,
    #[serde(rename = "client-id")]
    pub client_id: String,
    pub ip: String,
}
