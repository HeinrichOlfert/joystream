//! # Service discovery module
//! Service discovery module for the Joystream platform supports the storage providers.
//! It registers their 'pings' in the system with the expiration time, and stores the bootstrap
//! nodes for the Colossus.
//!
//! ## Comments
//!
//! Service discovery module uses bureaucracy module to authorize actions. It is generally used by
//! the Colossus service.
//!
//! ## Supported extrinsics
//!
//! - [set_ipns_id](./struct.Module.html#method.set_ipns_id) - Creates the AccountInfo to save an IPNS identity for the storage provider.
//! - [unset_ipns_id](./struct.Module.html#method.unset_ipns_id) - Deletes the AccountInfo with the IPNS identity for the storage provider.
//! - [set_default_lifetime](./struct.Module.html#method.set_default_lifetime) - Sets default lifetime for storage providers accounts info.
//! - [set_bootstrap_endpoints](./struct.Module.html#method.set_bootstrap_endpoints) - Sets bootstrap endpoints for the Colossus.
//!

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

mod mock;
mod tests;

use codec::{Decode, Encode};
use rstd::prelude::*;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use srml_support::{decl_event, decl_module, decl_storage, ensure};
use system::{self, ensure_root};
/*
  Although there is support for ed25519 keys as the IPNS identity key and we could potentially
  reuse the same key for the role account and ipns (and make this discovery module obselete)
  it is probably better to separate concerns.
  Why not to use a fixed size 32byte -> SHA256 hash of public key: because we would have to force
  specific key type on ipfs side.
  pub struct IPNSIdentity(pub [u8; 32]); // we loose the key type!
  pub type IPNSIdentity(pub u8, pub [u8; 32]); // we could add the keytype?
  can we use rust library in wasm runtime?
  https://github.com/multiformats/rust-multihash
  https://github.com/multiformats/multicodec/
  https://github.com/multiformats/multihash/
*/
/// base58 encoded IPNS identity multihash codec
pub type IPNSIdentity = Vec<u8>;

/// HTTP Url string to a discovery service endpoint
pub type Url = Vec<u8>;

// Alias for storage working group bureaucracy
pub(crate) type StorageBureaucracy<T> = bureaucracy::Module<T, bureaucracy::Instance2>;

/// Storage provider is a worker from the bureaucracy module.
pub type StorageProviderId<T> = bureaucracy::WorkerId<T>;

pub(crate) const MINIMUM_LIFETIME: u32 = 600; // 1hr assuming 6s block times
pub(crate) const DEFAULT_LIFETIME: u32 = MINIMUM_LIFETIME * 24; // 24hr

/// Defines the expiration date for the storage provider.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq)]
pub struct AccountInfo<BlockNumber> {
    /// IPNS Identity.
    pub identity: IPNSIdentity,
    /// Block at which information expires.
    pub expires_at: BlockNumber,
}

/// The _Service discovery_ main _Trait_.
pub trait Trait: system::Trait + bureaucracy::Trait<bureaucracy::Instance2> {
    /// _Service discovery_ event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_storage! {
    trait Store for Module<T: Trait> as Discovery {
        /// Bootstrap endpoints maintained by root
        pub BootstrapEndpoints get(bootstrap_endpoints): Vec<Url>;

        /// Mapping of service providers' storage provider id to their AccountInfo
        pub AccountInfoByStorageProviderId get(account_info_by_storage_provider_id):
            map StorageProviderId<T> => AccountInfo<T::BlockNumber>;

        /// Lifetime of an AccountInfo record in AccountInfoByAccountId map
        pub DefaultLifetime get(default_lifetime) config():
            T::BlockNumber = T::BlockNumber::from(DEFAULT_LIFETIME);
    }
}

decl_event! {
    /// _Service discovery_ events
    pub enum Event<T> where
        StorageProviderId = StorageProviderId<T>
       {
        /// Emits on updating of the account info.
        /// Params:
        /// - Id of the storage provider.
        /// - Id of the IPNS.
        AccountInfoUpdated(StorageProviderId, IPNSIdentity),

        /// Emits on removing of the account info.
        /// Params:
        /// - Id of the storage provider.
        AccountInfoRemoved(StorageProviderId),
    }
}

decl_module! {
    /// _Service discovery_ substrate module.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        /// Default deposit_event() handler
        fn deposit_event() = default;

        /// Creates the AccountInfo to save an IPNS identity for the storage provider.
        /// Requires signed storage provider credentials.
        pub fn set_ipns_id(
            origin,
            storage_provider_id: StorageProviderId<T>,
            id: Vec<u8>,
            lifetime: Option<T::BlockNumber>
        ) {
            <StorageBureaucracy<T>>::ensure_worker_signed(origin, &storage_provider_id)?;

            // TODO: ensure id is a valid base58 encoded IPNS identity

            let ttl = lifetime.map_or(Self::default_lifetime(), |value| {
                let min_lifetime = T::BlockNumber::from(MINIMUM_LIFETIME);
                if value >= min_lifetime {
                    value
                } else {
                    min_lifetime
                }
            });

            //
            // == MUTATION SAFE ==
            //

            <AccountInfoByStorageProviderId<T>>::insert(storage_provider_id, AccountInfo {
                identity: id.clone(),
                expires_at: <system::Module<T>>::block_number() + ttl,
            });

            Self::deposit_event(RawEvent::AccountInfoUpdated(storage_provider_id, id));
        }

        /// Deletes the AccountInfo with the IPNS identity for the storage provider.
        /// Requires signed storage provider credentials.
        pub fn unset_ipns_id(origin, storage_provider_id: StorageProviderId<T>) {
            <StorageBureaucracy<T>>::ensure_worker_signed(origin, &storage_provider_id)?;

            // == MUTATION SAFE ==

            if <AccountInfoByStorageProviderId<T>>::exists(storage_provider_id) {
                <AccountInfoByStorageProviderId<T>>::remove(storage_provider_id);
                Self::deposit_event(RawEvent::AccountInfoRemoved(storage_provider_id));
            }
        }

        // Privileged methods

        /// Sets default lifetime for storage providers accounts info. Requires root privileges.
        pub fn set_default_lifetime(origin, lifetime: T::BlockNumber) {
            ensure_root(origin)?;
            ensure!(lifetime >= T::BlockNumber::from(MINIMUM_LIFETIME),
                "discovery: default lifetime must be gte minimum lifetime");

            // == MUTATION SAFE ==

            <DefaultLifetime<T>>::put(lifetime);
        }

        /// Sets bootstrap endpoints for the Colossus. Requires root privileges.
        pub fn set_bootstrap_endpoints(origin, endpoints: Vec<Vec<u8>>) {
            ensure_root(origin)?;

            // == MUTATION SAFE ==

            BootstrapEndpoints::put(endpoints);
        }
    }
}

impl<T: Trait> Module<T> {
    /// Verifies that account info for the storage provider is still valid.
    pub fn is_account_info_expired(storage_provider_id: &StorageProviderId<T>) -> bool {
        !<AccountInfoByStorageProviderId<T>>::exists(storage_provider_id)
            || <system::Module<T>>::block_number()
                > <AccountInfoByStorageProviderId<T>>::get(storage_provider_id).expires_at
    }
}
