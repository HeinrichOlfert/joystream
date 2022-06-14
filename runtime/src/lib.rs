//! The Joystream Substrate Node runtime.

#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]
//Substrate internal issues.
#![allow(clippy::large_enum_variant)]
#![allow(clippy::unnecessary_mut_passed)]
#![allow(non_fmt_panics)]
#![allow(clippy::from_over_into)]

// Mutually exclusive feature check
#[cfg(all(feature = "staging_runtime", feature = "testing_runtime"))]
compile_error!("feature \"staging_runtime\" and feature \"testing_runtime\" cannot be enabled at the same time");

// Make the WASM binary available.
// This is required only by the node build.
// A dummy wasm_binary.rs will be built for the IDE.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

pub mod constants;
mod integration;
pub mod primitives;
mod proposals_configuration;
mod runtime_api;
#[cfg(test)]
mod tests;
/// Weights for pallets used in the runtime.
mod weights; // Runtime integration tests

#[macro_use]
extern crate lazy_static; // for proposals_configuration module

use frame_support::traits::{Currency, KeyOwnerProofSystem, LockIdentifier, OnUnbalanced, EnsureOneOf};
use frame_support::weights::{
    constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_PER_SECOND},
    DispatchClass, Weight,
};
use frame_support::weights::{WeightToFeeCoefficients, WeightToFeePolynomial};
use frame_support::{construct_runtime, parameter_types, PalletId};
use frame_system::limits::{BlockLength, BlockWeights};
use frame_system::{EnsureRoot, EnsureSigned};
use pallet_grandpa::{AuthorityId as GrandpaId, AuthorityList as GrandpaAuthorityList};
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use pallet_session::historical as pallet_session_historical;
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_core::crypto::KeyTypeId;
use sp_core::Hasher;
use sp_runtime::curve::PiecewiseLinear;
use sp_runtime::traits::{AccountIdLookup, BlakeTwo256, OpaqueKeys};
use sp_runtime::{create_runtime_str, generic, impl_opaque_keys, Perbill};
use sp_std::boxed::Box;
use sp_std::vec::Vec;
use sp_std::convert::{TryFrom, TryInto};

#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

pub use constants::*;
pub use primitives::*;
pub use proposals_configuration::*;
pub use runtime_api::*;

use integration::proposals::{CouncilManager, ExtrinsicProposalEncoder};

use common::working_group::{WorkingGroup, WorkingGroupBudgetHandler};
use council::ReferendumConnection;
use referendum::{CastVote, OptionResult};
use staking_handler::{LockComparator, StakingManager};

// Node dependencies
pub use common;
pub use council;
pub use forum;
pub use membership;

#[cfg(any(feature = "std", test))]
pub use pallet_balances::Call as BalancesCall;
pub use pallet_staking::StakerStatus;
pub use proposals_engine::ProposalParameters;
pub use referendum;
pub use working_group;

pub use content;
pub use content::MaxNumber;

/// This runtime version.
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("joystream-node"),
    impl_name: create_runtime_str!("joystream-node"),
    authoring_version: 11,
    spec_version: 20,
    impl_version: 0,
    apis: crate::runtime_api::EXPORTED_RUNTIME_API_VERSIONS,
    transaction_version: 1,
    state_version: 1,
};

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
    NativeVersion {
        runtime_version: VERSION,
        can_author_with: Default::default(),
    }
}

/// We assume that ~10% of the block weight is consumed by `on_initalize` handlers.
/// This is used to limit the maximal weight of a single extrinsic.
const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(10);
/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used
/// by  Operational  extrinsics.
const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);
/// We allow for 2 seconds of compute with a 12 second average block time.
pub const MAXIMUM_BLOCK_WEIGHT: Weight = WEIGHT_PER_SECOND * 2;

parameter_types! {
    pub const BlockHashCount: BlockNumber = 900; // mortal tx can be valid up to 1 hour after signing
    pub const Version: RuntimeVersion = VERSION;
    pub RuntimeBlockLength: BlockLength =
        BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
    pub RuntimeBlockWeights: BlockWeights = BlockWeights::builder()
        .base_block(BlockExecutionWeight::get())
        .for_class(DispatchClass::all(), |weights| {
            weights.base_extrinsic = ExtrinsicBaseWeight::get();
        })
        .for_class(DispatchClass::Normal, |weights| {
            weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
        })
        .for_class(DispatchClass::Operational, |weights| {
            weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
            // Operational transactions have some extra reserved space, so that they
            // are included even if block reached `MAXIMUM_BLOCK_WEIGHT`.
            weights.reserved = Some(
                MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT
            );
        })
        .avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
        .build_or_panic();
    pub const SS58Prefix: u8 = 42; // Ss58AddressFormat::SubstrateAccount
}

// TODO: We need to adjust weight of this pallet
// once we move to a newer version of substrate where parameters
// are not discarded. See the comment in 'scripts/generate-weights.sh'
impl frame_system::Config for Runtime {
    /// The basic call filter to use in dispatchable.
    type BaseCallFilter = ();
    /// Block & extrinsics weights: base values and limits.
    type BlockWeights = RuntimeBlockWeights;
    /// The maximum length of a block (in bytes).
    type BlockLength = RuntimeBlockLength;
    /// The identifier used to distinguish between accounts.
    type AccountId = AccountId;
    /// The aggregated dispatch type that is available for extrinsics.
    type Call = Call;
    /// The lookup mechanism to get account ID from whatever is passed in dispatchers.
    type Lookup = AccountIdLookup<AccountId, ()>;
    /// The index type for storing how many extrinsics an account has signed.
    type Index = Index;
    /// The index type for blocks.
    type BlockNumber = BlockNumber;
    /// The type for hashing blocks and tries.
    type Hash = Hash;
    /// The hashing algorithm used.
    type Hashing = BlakeTwo256;
    /// The header type.
    type Header = generic::Header<BlockNumber, BlakeTwo256>;
    /// The ubiquitous event type.
    type Event = Event;
    /// The ubiquitous origin type.
    type Origin = Origin;
    /// Maximum number of block number to block hash mappings to keep (oldest pruned first).
    type BlockHashCount = BlockHashCount;
    /// The weight of database operations that the runtime can invoke.
    type DbWeight = RocksDbWeight;
    /// Version of the runtime.
    type Version = Version;
    /// Converts a Pallet to the index of the Pallet in `construct_runtime!`.
    ///
    /// This type is being generated by `construct_runtime!`.
    type PalletInfo = PalletInfo;
    /// What to do if a new account is created.
    type OnNewAccount = ();
    /// What to do if an account is fully reaped from the system.
    type OnKilledAccount = ();
    /// The data to be stored in an account.
    type AccountData = pallet_balances::AccountData<Balance>;
    /// Weight information for the extrinsics of this pallet.
    type SystemWeightInfo = ();
    /// This is used as an identifier of the chain. 42 is the generic substrate prefix.
    type SS58Prefix = SS58Prefix;
}

impl substrate_utility::Config for Runtime {
    type Event = Event;
    type Call = Call;
    type WeightInfo = weights::substrate_utility::WeightInfo;
}

parameter_types! {
    pub const EpochDuration: u64 = EPOCH_DURATION_IN_SLOTS as u64;
    pub const ReportLongevity: u64 =
            BondingDuration::get() as u64 * SessionsPerEra::get() as u64 *
    EpochDuration::get();
    pub const ExpectedBlockTime: Moment = MILLISECS_PER_BLOCK;
}

impl pallet_babe::Config for Runtime {
    type EpochDuration = EpochDuration;
    type ExpectedBlockTime = ExpectedBlockTime;
    type EpochChangeTrigger = pallet_babe::ExternalTrigger;
    type KeyOwnerProofSystem = Historical;

    type KeyOwnerProof = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
        KeyTypeId,
        pallet_babe::AuthorityId,
    )>>::Proof;

    type KeyOwnerIdentification = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
        KeyTypeId,
        pallet_babe::AuthorityId,
    )>>::IdentificationTuple;

    type HandleEquivocation =
        pallet_babe::EquivocationHandler<Self::KeyOwnerIdentification, Offences, ReportLongevity>;

    type WeightInfo = ();
}

impl pallet_grandpa::Config for Runtime {
    type Event = Event;
    type Call = Call;
    type KeyOwnerProof =
        <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::Proof;

    type KeyOwnerIdentification = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
        KeyTypeId,
        GrandpaId,
    )>>::IdentificationTuple;

    type KeyOwnerProofSystem = Historical;

    type HandleEquivocation = pallet_grandpa::EquivocationHandler<
        Self::KeyOwnerIdentification,
        Offences,
        ReportLongevity,
    >;
    type WeightInfo = ();
}

impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Runtime
where
    Call: From<LocalCall>,
{
    fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
        call: Call,
        public: <Signature as sp_runtime::traits::Verify>::Signer,
        account: AccountId,
        nonce: Index,
    ) -> Option<(
        Call,
        <UncheckedExtrinsic as sp_runtime::traits::Extrinsic>::SignaturePayload,
    )> {
        integration::transactions::create_transaction::<C>(call, public, account, nonce)
    }
}

impl frame_system::offchain::SigningTypes for Runtime {
    type Public = <Signature as sp_runtime::traits::Verify>::Signer;
    type Signature = Signature;
}

impl<C> frame_system::offchain::SendTransactionTypes<C> for Runtime
where
    Call: From<C>,
{
    type Extrinsic = UncheckedExtrinsic;
    type OverarchingCall = Call;
}

parameter_types! {
    pub const MinimumPeriod: Moment = SLOT_DURATION / 2;
}

impl pallet_timestamp::Config for Runtime {
    type Moment = Moment;
    type OnTimestampSet = Babe;
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = weights::pallet_timestamp::WeightInfo;
}

parameter_types! {
    pub const MaxLocks: u32 = 50;
}

impl pallet_balances::Config for Runtime {
    type Balance = Balance;
    type DustRemoval = ();
    type Event = Event;
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = weights::pallet_balances::WeightInfo;
    type MaxLocks = MaxLocks;
}

parameter_types! {
    pub const TransactionByteFee: Balance = 0;
}

// Temporary commented for Olympia: https://github.com/Joystream/joystream/issues/3237
// TODO: Restore after the Olympia release
// parameter_types! {
//     pub const TransactionByteFee: Balance = 1;
// }

type NegativeImbalance = <Balances as Currency<AccountId>>::NegativeImbalance;

pub struct Author;
impl OnUnbalanced<NegativeImbalance> for Author {
    fn on_nonzero_unbalanced(amount: NegativeImbalance) {
        Balances::resolve_creating(&Authorship::author(), amount);
    }
}
// Temporary commented for Olympia: https://github.com/Joystream/joystream/issues/3237
// TODO: Restore after the Olympia release
// pub struct DealWithFees;
// impl OnUnbalanced<NegativeImbalance> for DealWithFees {
//     fn on_unbalanceds<B>(mut fees_then_tips: impl Iterator<Item = NegativeImbalance>) {
//         if let Some(fees) = fees_then_tips.next() {
//             // for fees, 20% to author, for now we don't have treasury so the 80% is ignored
//             let mut split = fees.ration(80, 20);
//             if let Some(tips) = fees_then_tips.next() {
//                 // For tips %100 are for the author
//                 tips.ration_merge_into(0, 100, &mut split);
//             }
//             Author::on_unbalanced(split.1);
//         }
//     }
// }

/// Stub for zero transaction weights.
pub struct NoWeights;
impl WeightToFeePolynomial for NoWeights {
    type Balance = Balance;
}

impl pallet_transaction_payment::Config for Runtime {
    type OnChargeTransaction = pallet_transaction_payment::CurrencyAdapter<Balances, ()>;
    type TransactionByteFee = TransactionByteFee;
    type WeightToFee = NoWeights;
    type FeeMultiplierUpdate = ();
}

impl pallet_sudo::Config for Runtime {
    type Event = Event;
    type Call = Call;
}

parameter_types! {
    pub const UncleGenerations: BlockNumber = 0;
}

impl pallet_authorship::Config for Runtime {
    type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Babe>;
    type UncleGenerations = UncleGenerations;
    type FilterUncle = ();
    type EventHandler = (Staking, ImOnline);
}

impl_opaque_keys! {
    pub struct SessionKeys {
        pub grandpa: Grandpa,
        pub babe: Babe,
        pub im_online: ImOnline,
        pub authority_discovery: AuthorityDiscovery,
    }
}
// NOTE: `SessionHandler` and `SessionKeys` are co-dependent: One key will be used for each handler.
// The number and order of items in `SessionHandler` *MUST* be the same number and order of keys in
// `SessionKeys`.
// TODO: Introduce some structure to tie these together to make it a bit less of a footgun. This
// should be easy, since OneSessionHandler trait provides the `Key` as an associated type. #2858
parameter_types! {
    pub const DisabledValidatorsThreshold: Perbill = Perbill::from_percent(17);
}

impl pallet_session::Config for Runtime {
    type Event = Event;
    type ValidatorId = AccountId;
    type ValidatorIdOf = pallet_staking::StashOf<Self>;
    type ShouldEndSession = Babe;
    type NextSessionRotation = Babe;
    type SessionManager = pallet_session::historical::NoteHistoricalRoot<Self, Staking>;
    type SessionHandler = <SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
    type Keys = SessionKeys;
    type DisabledValidatorsThreshold = DisabledValidatorsThreshold;
    type WeightInfo = weights::pallet_session::WeightInfo;
}

impl pallet_session::historical::Config for Runtime {
    type FullIdentification = pallet_staking::Exposure<AccountId, Balance>;
    type FullIdentificationOf = pallet_staking::ExposureOf<Runtime>;
}

pallet_staking_reward_curve::build! {
    const REWARD_CURVE: PiecewiseLinear<'static> = curve!(
        min_inflation: 0_050_000,
        max_inflation: 0_180_000,
        ideal_stake: 0_300_000,
        falloff: 0_050_000,
        max_piece_count: 100,
        test_precision: 0_005_000,
    );
}

parameter_types! {
    pub const SessionDuration: BlockNumber = EPOCH_DURATION_IN_SLOTS as _;
    pub const ImOnlineUnsignedPriority: TransactionPriority = TransactionPriority::max_value();
    /// We prioritize im-online heartbeats over election solution submission.
    pub const StakingUnsignedPriority: TransactionPriority = TransactionPriority::max_value() / 2;
}

parameter_types! {
    pub const SessionsPerEra: sp_staking::SessionIndex = 6;
    pub const BondingDuration: pallet_staking::EraIndex = BONDING_DURATION;
    pub const SlashDeferDuration: pallet_staking::EraIndex = BONDING_DURATION - 1; // 'slightly less' than the bonding duration.
    pub const RewardCurve: &'static PiecewiseLinear<'static> = &REWARD_CURVE;
    pub const MaxNominatorRewardedPerValidator: u32 = 64;
    pub const ElectionLookahead: BlockNumber = EPOCH_DURATION_IN_BLOCKS / 4;
    pub const MaxIterations: u32 = 10;
    // 0.05%. The higher the value, the more strict solution acceptance becomes.
    pub MinSolutionScoreBump: Perbill = Perbill::from_rational_approximation(5u32, 10_000);
}

impl pallet_staking::Config for Runtime {
    type Currency = Balances;
    type UnixTime = Timestamp;
    type CurrencyToVote = frame_support::traits::SaturatingCurrencyToVote;
    type RewardRemainder = (); // Could be Treasury.
    type Event = Event;
    type Slash = (); // Where to send the slashed funds. Could be Treasury.
    type Reward = (); // Rewards are minted from the void.
    type SessionsPerEra = SessionsPerEra;
    type BondingDuration = BondingDuration;
    type SlashDeferDuration = SlashDeferDuration;
    type SlashCancelOrigin = EnsureRoot<AccountId>; // Requires sudo. Parity recommends: a super-majority of the council can cancel the slash.
    type SessionInterface = Self;
    type RewardCurve = RewardCurve;
    type NextNewSession = Session;
    type ElectionLookahead = ElectionLookahead;
    type Call = Call;
    type MaxIterations = MaxIterations;
    type MinSolutionScoreBump = MinSolutionScoreBump;
    type MaxNominatorRewardedPerValidator = MaxNominatorRewardedPerValidator;
    type UnsignedPriority = StakingUnsignedPriority;
    type WeightInfo = weights::pallet_staking::WeightInfo;
    type OffchainSolutionWeightLimit = ();
}

impl pallet_im_online::Config for Runtime {
    type AuthorityId = ImOnlineId;
    type Event = Event;
    type ValidatorSet = Historical;
    type SessionDuration = SessionDuration;
    type ReportUnresponsiveness = Offences;
    // Using the default weights until we check if we can run the benchmarks for this pallet in
    // the reference machine in an acceptable time.
    type WeightInfo = ();
    type UnsignedPriority = ImOnlineUnsignedPriority;
}

parameter_types! {
    pub OffencesWeightSoftLimit: Weight = Perbill::from_percent(60) * MAXIMUM_BLOCK_WEIGHT;
}

impl pallet_offences::Config for Runtime {
    type Event = Event;
    type IdentificationTuple = pallet_session::historical::IdentificationTuple<Self>;
    type OnOffenceHandler = Staking;
    type WeightSoftLimit = OffencesWeightSoftLimit;
}

impl pallet_authority_discovery::Config for Runtime {}

parameter_types! {
    pub const WindowSize: BlockNumber = 101;
    pub const ReportLatency: BlockNumber = 1000;
}

// type EntityId = <Runtime as content_directory::Config>::EntityId;

parameter_types! {
    pub const MaxNumberOfCuratorsPerGroup: MaxNumber = 50;
    pub const MaxModerators: u64 = 5;    // TODO: update
    pub const CleanupMargin: u32 = 3;    // TODO: update
    pub const CleanupCost: u32 = 1; // TODO: update
    pub const PricePerByte: u32 = 2; // TODO: update
    pub const ContentModuleId: PalletId = PalletId(*b"mContent"); // module content
    pub const BloatBondCap: u32 = 1000;  // TODO: update
}

impl content::Config for Runtime {
    type Event = Event;
    type ChannelCategoryId = ChannelCategoryId;
    type VideoId = VideoId;
    type OpenAuctionId = OpenAuctionId;
    type VideoCategoryId = VideoCategoryId;
    type MaxNumberOfCuratorsPerGroup = MaxNumberOfCuratorsPerGroup;
    type DataObjectStorage = Storage;
    type VideoPostId = VideoPostId;
    type ReactionId = ReactionId;
    type MaxModerators = MaxModerators;
    type PricePerByte = PricePerByte;
    type BloatBondCap = BloatBondCap;
    type CleanupMargin = CleanupMargin;
    type CleanupCost = CleanupCost;
    type ModuleId = ContentModuleId;
    type MemberAuthenticator = Members;
}

// The referendum instance alias.
pub type ReferendumInstance = referendum::Instance1;
pub type ReferendumModule = referendum::Module<Runtime, ReferendumInstance>;
pub type CouncilModule = council::Module<Runtime>;

// Production coucil and elections configuration
#[cfg(not(any(feature = "staging_runtime", feature = "testing_runtime")))]
parameter_types! {
    // referendum parameters
    pub const MaxSaltLength: u64 = 32;
    pub const VoteStageDuration: BlockNumber = 14400;
    pub const RevealStageDuration: BlockNumber = 14400;
    pub const MinimumVotingStake: u64 = 10000;

    // council parameteres
    pub const MinNumberOfExtraCandidates: u64 = 1;
    pub const AnnouncingPeriodDuration: BlockNumber = 14400;
    pub const IdlePeriodDuration: BlockNumber = 57600;
    pub const CouncilSize: u64 = 5;
    pub const MinCandidateStake: u64 = 11000;
    pub const ElectedMemberRewardPeriod: BlockNumber = 14400;
    pub const DefaultBudgetIncrement: u64 = 5000000;
    pub const BudgetRefillPeriod: BlockNumber = 14400;
    pub const MaxWinnerTargetCount: u64 = 10; // should be greater than council size
}

// Common staging and playground coucil and elections configuration
// CouncilSize is defined separately
#[cfg(feature = "staging_runtime")]
parameter_types! {
    // referendum parameters
    pub const MaxSaltLength: u64 = 32;
    pub const VoteStageDuration: BlockNumber = 100;
    pub const RevealStageDuration: BlockNumber = 50;
    pub const MinimumVotingStake: u64 = 10000;

    // council parameteres
    pub const MinNumberOfExtraCandidates: u64 = 1;
    pub const AnnouncingPeriodDuration: BlockNumber = 200;
    pub const IdlePeriodDuration: BlockNumber = 400;
    pub const MinCandidateStake: u64 = 11000;
    pub const ElectedMemberRewardPeriod: BlockNumber = 14400;
    pub const DefaultBudgetIncrement: u64 = 10000000;
    pub const BudgetRefillPeriod: BlockNumber = 1000;
    pub const MaxWinnerTargetCount: u64 = 10;
}

// Staging council size
#[cfg(feature = "staging_runtime")]
#[cfg(not(feature = "playground_runtime"))]
parameter_types! {
    pub const CouncilSize: u64 = 5;
}

// Playground council size
#[cfg(feature = "staging_runtime")]
#[cfg(feature = "playground_runtime")]
parameter_types! {
    pub const CouncilSize: u64 = 1;
}

// Testing config
#[cfg(feature = "testing_runtime")]
parameter_types! {
    // referendum parameters
    pub const MaxSaltLength: u64 = 32;
    pub const VoteStageDuration: BlockNumber = 20;
    pub const RevealStageDuration: BlockNumber = 20;
    pub const MinimumVotingStake: u64 = 10000;

    // council parameteres
    pub const MinNumberOfExtraCandidates: u64 = 1;
    pub const AnnouncingPeriodDuration: BlockNumber = 20;
    pub const IdlePeriodDuration: BlockNumber = 20;
    pub const CouncilSize: u64 = 5;
    pub const MinCandidateStake: u64 = 11000;
    pub const ElectedMemberRewardPeriod: BlockNumber = 14400;
    pub const DefaultBudgetIncrement: u64 = 10000000;
    pub const BudgetRefillPeriod: BlockNumber = 1000;
    pub const MaxWinnerTargetCount: u64 = 10;
}

impl referendum::Config<ReferendumInstance> for Runtime {
    type Event = Event;
    type MaxSaltLength = MaxSaltLength;
    type StakingHandler = VotingStakingManager;
    type ManagerOrigin =
        EnsureOneOf<EnsureSigned<Self::AccountId>, EnsureRoot<Self::AccountId>>;
    type VotePower = Balance;
    type VoteStageDuration = VoteStageDuration;
    type RevealStageDuration = RevealStageDuration;
    type MinimumStake = MinimumVotingStake;
    type WeightInfo = weights::referendum::WeightInfo;
    type MaxWinnerTargetCount = MaxWinnerTargetCount;

    fn calculate_vote_power(
        _account_id: &<Self as frame_system::Config>::AccountId,
        stake: &Balance,
    ) -> Self::VotePower {
        *stake
    }

    fn can_unlock_vote_stake(vote: &CastVote<Self::Hash, Balance, Self::MemberId>) -> bool {
        <CouncilModule as ReferendumConnection<Runtime>>::can_unlock_vote_stake(vote).is_ok()
    }

    fn process_results(winners: &[OptionResult<Self::MemberId, Self::VotePower>]) {
        let tmp_winners: Vec<OptionResult<Self::MemberId, Self::VotePower>> = winners
            .iter()
            .map(|item| OptionResult {
                option_id: item.option_id,
                vote_power: item.vote_power,
            })
            .collect();
        <CouncilModule as ReferendumConnection<Runtime>>::recieve_referendum_results(
            tmp_winners.as_slice(),
        );
    }

    fn is_valid_option_id(option_index: &u64) -> bool {
        <CouncilModule as ReferendumConnection<Runtime>>::is_valid_candidate_id(option_index)
    }

    fn get_option_power(option_id: &u64) -> Self::VotePower {
        <CouncilModule as ReferendumConnection<Runtime>>::get_option_power(option_id)
    }

    fn increase_option_power(option_id: &u64, amount: &Self::VotePower) {
        <CouncilModule as ReferendumConnection<Runtime>>::increase_option_power(option_id, amount);
    }
}

impl council::Config for Runtime {
    type Event = Event;
    type Referendum = ReferendumModule;
    type MinNumberOfExtraCandidates = MinNumberOfExtraCandidates;
    type CouncilSize = CouncilSize;
    type AnnouncingPeriodDuration = AnnouncingPeriodDuration;
    type IdlePeriodDuration = IdlePeriodDuration;
    type MinCandidateStake = MinCandidateStake;
    type CandidacyLock = StakingManager<Self, CandidacyLockId>;
    type CouncilorLock = StakingManager<Self, CouncilorLockId>;
    type StakingAccountValidator = Members;
    type ElectedMemberRewardPeriod = ElectedMemberRewardPeriod;
    type BudgetRefillPeriod = BudgetRefillPeriod;
    type MemberOriginValidator = Members;
    type WeightInfo = weights::council::WeightInfo;

    fn new_council_elected(_elected_members: &[council::CouncilMemberOf<Self>]) {
        <proposals_engine::Module<Runtime>>::reject_active_proposals();
        <proposals_engine::Module<Runtime>>::reactivate_pending_constitutionality_proposals();
    }
}

impl common::StorageOwnership for Runtime {
    type ChannelId = ChannelId;
    type ContentId = ContentId;
    type DataObjectTypeId = DataObjectTypeId;
}

parameter_types! {
    pub const MaxDistributionBucketFamilyNumber: u64 = 200;
    pub const DataObjectDeletionPrize: Balance = 0; //TODO: Change during Olympia release
    pub const BlacklistSizeLimit: u64 = 10000; //TODO: adjust value
    pub const MaxRandomIterationNumber: u64 = 10; //TODO: adjust value
    pub const MaxNumberOfPendingInvitationsPerDistributionBucket: u64 = 20; //TODO: adjust value
    pub const StorageModuleId: PalletId = PalletId(*b"mstorage"); // module storage
    pub const StorageBucketsPerBagValueConstraint: storage::StorageBucketsPerBagValueConstraint =
        storage::StorageBucketsPerBagValueConstraint {min: 5, max_min_diff: 15}; //TODO: adjust value
    pub const DefaultMemberDynamicBagNumberOfStorageBuckets: u64 = 5; //TODO: adjust value
    pub const DefaultChannelDynamicBagNumberOfStorageBuckets: u64 = 5; //TODO: adjust value
    pub const DistributionBucketsPerBagValueConstraint: storage::DistributionBucketsPerBagValueConstraint =
        storage::DistributionBucketsPerBagValueConstraint {min: 1, max_min_diff: 100}; //TODO: adjust value
    pub const MaxDataObjectSize: u64 = 10 * 1024 * 1024 * 1024; // 10 GB
}

impl storage::Config for Runtime {
    type Event = Event;
    type DataObjectId = DataObjectId;
    type StorageBucketId = StorageBucketId;
    type DistributionBucketIndex = DistributionBucketIndex;
    type DistributionBucketFamilyId = DistributionBucketFamilyId;
    type ChannelId = ChannelId;
    type DataObjectDeletionPrize = DataObjectDeletionPrize;
    type BlacklistSizeLimit = BlacklistSizeLimit;
    type ModuleId = StorageModuleId;
    type StorageBucketsPerBagValueConstraint = StorageBucketsPerBagValueConstraint;
    type DefaultMemberDynamicBagNumberOfStorageBuckets =
        DefaultMemberDynamicBagNumberOfStorageBuckets;
    type DefaultChannelDynamicBagNumberOfStorageBuckets =
        DefaultChannelDynamicBagNumberOfStorageBuckets;
    type Randomness = RandomnessCollectiveFlip;
    type MaxRandomIterationNumber = MaxRandomIterationNumber;
    type MaxDistributionBucketFamilyNumber = MaxDistributionBucketFamilyNumber;
    type DistributionBucketsPerBagValueConstraint = DistributionBucketsPerBagValueConstraint;
    type DistributionBucketOperatorId = DistributionBucketOperatorId;
    type MaxNumberOfPendingInvitationsPerDistributionBucket =
        MaxNumberOfPendingInvitationsPerDistributionBucket;
    type MaxDataObjectSize = MaxDataObjectSize;
    type ContentId = ContentId;
    type StorageWorkingGroup = StorageWorkingGroup;
    type DistributionWorkingGroup = DistributionWorkingGroup;
}

impl common::membership::MembershipTypes for Runtime {
    type MemberId = MemberId;
    type ActorId = ActorId;
}

parameter_types! {
    pub const DefaultMembershipPrice: Balance = 100;
    pub const ReferralCutMaximumPercent: u8 = 50;
    pub const DefaultInitialInvitationBalance: Balance = 100;
    // The candidate stake should be more than the transaction fee which currently is 53
    pub const CandidateStake: Balance = 200;
}

impl membership::Config for Runtime {
    type Event = Event;
    type DefaultMembershipPrice = DefaultMembershipPrice;
    type DefaultInitialInvitationBalance = DefaultInitialInvitationBalance;
    type InvitedMemberStakingHandler = InvitedMemberStakingManager;
    type StakingCandidateStakingHandler = BoundStakingAccountStakingManager;
    type WorkingGroup = MembershipWorkingGroup;
    type WeightInfo = weights::membership::WeightInfo;
    type ReferralCutMaximumPercent = ReferralCutMaximumPercent;
    type CandidateStake = CandidateStake;
}

parameter_types! {
    pub const MaxCategoryDepth: u64 = 6;
    pub const MaxSubcategories: u64 = 40;
    pub const MaxThreadsInCategory: u64 = 20;
    pub const MaxPostsInThread: u64 = 20;
    pub const MaxModeratorsForCategory: u64 = 20;
    pub const MaxCategories: u64 = 40;
    pub const MaxPollAlternativesNumber: u64 = 20;
    pub const ThreadDeposit: u64 = 30;
    pub const PostDeposit: u64 = 10;
    pub const ForumModuleId: PalletId = PalletId(*b"mo:forum"); // module : forum
    pub const PostLifeTime: BlockNumber = 3600;
}

pub struct MapLimits;
impl forum::StorageLimits for MapLimits {
    type MaxSubcategories = MaxSubcategories;
    type MaxModeratorsForCategory = MaxModeratorsForCategory;
    type MaxCategories = MaxCategories;
    type MaxPollAlternativesNumber = MaxPollAlternativesNumber;
}

impl forum::Config for Runtime {
    type Event = Event;
    type ThreadId = ThreadId;
    type PostId = PostId;
    type CategoryId = u64;
    type PostReactionId = u64;
    type MaxCategoryDepth = MaxCategoryDepth;
    type ThreadDeposit = ThreadDeposit;
    type PostDeposit = PostDeposit;
    type ModuleId = ForumModuleId;
    type MapLimits = MapLimits;
    type WeightInfo = weights::forum::WeightInfo;
    type WorkingGroup = ForumWorkingGroup;
    type MemberOriginValidator = Members;
    type PostLifeTime = PostLifeTime;

    fn calculate_hash(text: &[u8]) -> Self::Hash {
        Self::Hashing::hash(text)
    }
}

impl LockComparator<<Runtime as pallet_balances::Config>::Balance> for Runtime {
    fn are_locks_conflicting(new_lock: &LockIdentifier, existing_locks: &[LockIdentifier]) -> bool {
        let other_locks_present = !existing_locks.is_empty();
        let new_lock_is_rivalrous = !NON_RIVALROUS_LOCKS.contains(new_lock);
        let existing_locks_contain_rivalrous_lock = existing_locks
            .iter()
            .any(|lock_id| !NON_RIVALROUS_LOCKS.contains(lock_id));

        other_locks_present && new_lock_is_rivalrous && existing_locks_contain_rivalrous_lock
    }
}

parameter_types! {
    pub const MaxWorkerNumberLimit: u32 = 100;
    pub const MinUnstakingPeriodLimit: u32 = 43200;
    pub const ForumWorkingGroupRewardPeriod: u32 = 14400 + 10;
    pub const StorageWorkingGroupRewardPeriod: u32 = 14400 + 20;
    pub const ContentWorkingGroupRewardPeriod: u32 = 14400 + 30;
    pub const MembershipRewardPeriod: u32 = 14400 + 40;
    pub const GatewayRewardPeriod: u32 = 14400 + 50;
    pub const OperationsAlphaRewardPeriod: u32 = 14400 + 60;
    pub const OperationsBetaRewardPeriod: u32 = 14400 + 70;
    pub const OperationsGammaRewardPeriod: u32 = 14400 + 80;
    pub const DistributionRewardPeriod: u32 = 14400 + 90;
    // This should be more costly than `apply_on_opening` fee with the current configuration
    // the base cost of `apply_on_opening` in tokens is 193. And has a very slight slope
    // with the lenght with the length of rationale, with 2000 stake we are probably safe.
    pub const MinimumApplicationStake: Balance = 2000;
    // This should be more costly than `add_opening` fee with the current configuration
    // the base cost of `add_opening` in tokens is 81. And has a very slight slope
    // with the lenght with the length of rationale, with 2000 stake we are probably safe.
    pub const LeaderOpeningStake: Balance = 2000;
}

// Staking managers type aliases.
pub type ForumWorkingGroupStakingManager =
    staking_handler::StakingManager<Runtime, ForumGroupLockId>;
pub type VotingStakingManager = staking_handler::StakingManager<Runtime, VotingLockId>;
pub type ContentWorkingGroupStakingManager =
    staking_handler::StakingManager<Runtime, ContentWorkingGroupLockId>;
pub type StorageWorkingGroupStakingManager =
    staking_handler::StakingManager<Runtime, StorageWorkingGroupLockId>;
pub type MembershipWorkingGroupStakingManager =
    staking_handler::StakingManager<Runtime, MembershipWorkingGroupLockId>;
pub type InvitedMemberStakingManager =
    staking_handler::StakingManager<Runtime, InvitedMemberLockId>;
pub type BoundStakingAccountStakingManager =
    staking_handler::StakingManager<Runtime, BoundStakingAccountLockId>;
pub type GatewayWorkingGroupStakingManager =
    staking_handler::StakingManager<Runtime, GatewayWorkingGroupLockId>;
pub type OperationsWorkingGroupAlphaStakingManager =
    staking_handler::StakingManager<Runtime, OperationsWorkingGroupAlphaLockId>;
pub type OperationsWorkingGroupBetaStakingManager =
    staking_handler::StakingManager<Runtime, OperationsWorkingGroupBetaLockId>;
pub type OperationsWorkingGroupGammaStakingManager =
    staking_handler::StakingManager<Runtime, OperationsWorkingGroupGammaLockId>;
pub type DistributionWorkingGroupStakingManager =
    staking_handler::StakingManager<Runtime, DistributionWorkingGroupLockId>;

// The forum working group instance alias.
pub type ForumWorkingGroupInstance = working_group::Instance1;

// The storage working group instance alias.
pub type StorageWorkingGroupInstance = working_group::Instance2;

// The content directory working group instance alias.
pub type ContentWorkingGroupInstance = working_group::Instance3;

// The builder working group instance alias.
pub type OperationsWorkingGroupInstanceAlpha = working_group::Instance4;

// The gateway working group instance alias.
pub type GatewayWorkingGroupInstance = working_group::Instance5;

// The membership working group instance alias.
pub type MembershipWorkingGroupInstance = working_group::Instance6;

// The builder working group instance alias.
pub type OperationsWorkingGroupInstanceBeta = working_group::Instance7;

// The builder working group instance alias.
pub type OperationsWorkingGroupInstanceGamma = working_group::Instance8;

// The distribution working group instance alias.
pub type DistributionWorkingGroupInstance = working_group::Instance9;

impl working_group::Config<ForumWorkingGroupInstance> for Runtime {
    type Event = Event;
    type MaxWorkerNumberLimit = MaxWorkerNumberLimit;
    type StakingHandler = ForumWorkingGroupStakingManager;
    type StakingAccountValidator = Members;
    type MemberOriginValidator = Members;
    type MinUnstakingPeriodLimit = MinUnstakingPeriodLimit;
    type RewardPeriod = ForumWorkingGroupRewardPeriod;
    type WeightInfo = weights::working_group::WeightInfo;
    type MinimumApplicationStake = MinimumApplicationStake;
    type LeaderOpeningStake = LeaderOpeningStake;
}

impl working_group::Config<StorageWorkingGroupInstance> for Runtime {
    type Event = Event;
    type MaxWorkerNumberLimit = MaxWorkerNumberLimit;
    type StakingHandler = StorageWorkingGroupStakingManager;
    type StakingAccountValidator = Members;
    type MemberOriginValidator = Members;
    type MinUnstakingPeriodLimit = MinUnstakingPeriodLimit;
    type RewardPeriod = StorageWorkingGroupRewardPeriod;
    type WeightInfo = weights::working_group::WeightInfo;
    type MinimumApplicationStake = MinimumApplicationStake;
    type LeaderOpeningStake = LeaderOpeningStake;
}

impl working_group::Config<ContentWorkingGroupInstance> for Runtime {
    type Event = Event;
    type MaxWorkerNumberLimit = MaxWorkerNumberLimit;
    type StakingHandler = ContentWorkingGroupStakingManager;
    type StakingAccountValidator = Members;
    type MemberOriginValidator = Members;
    type MinUnstakingPeriodLimit = MinUnstakingPeriodLimit;
    type RewardPeriod = ContentWorkingGroupRewardPeriod;
    type WeightInfo = weights::working_group::WeightInfo;
    type MinimumApplicationStake = MinimumApplicationStake;
    type LeaderOpeningStake = LeaderOpeningStake;
}

impl working_group::Config<MembershipWorkingGroupInstance> for Runtime {
    type Event = Event;
    type MaxWorkerNumberLimit = MaxWorkerNumberLimit;
    type StakingHandler = MembershipWorkingGroupStakingManager;
    type StakingAccountValidator = Members;
    type MemberOriginValidator = Members;
    type MinUnstakingPeriodLimit = MinUnstakingPeriodLimit;
    type RewardPeriod = MembershipRewardPeriod;
    type WeightInfo = weights::working_group::WeightInfo;
    type MinimumApplicationStake = MinimumApplicationStake;
    type LeaderOpeningStake = LeaderOpeningStake;
}

impl working_group::Config<OperationsWorkingGroupInstanceAlpha> for Runtime {
    type Event = Event;
    type MaxWorkerNumberLimit = MaxWorkerNumberLimit;
    type StakingHandler = OperationsWorkingGroupAlphaStakingManager;
    type StakingAccountValidator = Members;
    type MemberOriginValidator = Members;
    type MinUnstakingPeriodLimit = MinUnstakingPeriodLimit;
    type RewardPeriod = OperationsAlphaRewardPeriod;
    type WeightInfo = weights::working_group::WeightInfo;
    type MinimumApplicationStake = MinimumApplicationStake;
    type LeaderOpeningStake = LeaderOpeningStake;
}

impl working_group::Config<GatewayWorkingGroupInstance> for Runtime {
    type Event = Event;
    type MaxWorkerNumberLimit = MaxWorkerNumberLimit;
    type StakingHandler = GatewayWorkingGroupStakingManager;
    type StakingAccountValidator = Members;
    type MemberOriginValidator = Members;
    type MinUnstakingPeriodLimit = MinUnstakingPeriodLimit;
    type RewardPeriod = GatewayRewardPeriod;
    type WeightInfo = weights::working_group::WeightInfo;
    type MinimumApplicationStake = MinimumApplicationStake;
    type LeaderOpeningStake = LeaderOpeningStake;
}

impl working_group::Config<OperationsWorkingGroupInstanceBeta> for Runtime {
    type Event = Event;
    type MaxWorkerNumberLimit = MaxWorkerNumberLimit;
    type StakingHandler = OperationsWorkingGroupBetaStakingManager;
    type StakingAccountValidator = Members;
    type MemberOriginValidator = Members;
    type MinUnstakingPeriodLimit = MinUnstakingPeriodLimit;
    type RewardPeriod = OperationsBetaRewardPeriod;
    type WeightInfo = weights::working_group::WeightInfo;
    type MinimumApplicationStake = MinimumApplicationStake;
    type LeaderOpeningStake = LeaderOpeningStake;
}

impl working_group::Config<OperationsWorkingGroupInstanceGamma> for Runtime {
    type Event = Event;
    type MaxWorkerNumberLimit = MaxWorkerNumberLimit;
    type StakingHandler = OperationsWorkingGroupGammaStakingManager;
    type StakingAccountValidator = Members;
    type MemberOriginValidator = Members;
    type MinUnstakingPeriodLimit = MinUnstakingPeriodLimit;
    type RewardPeriod = OperationsGammaRewardPeriod;
    type WeightInfo = weights::working_group::WeightInfo;
    type MinimumApplicationStake = MinimumApplicationStake;
    type LeaderOpeningStake = LeaderOpeningStake;
}

impl working_group::Config<DistributionWorkingGroupInstance> for Runtime {
    type Event = Event;
    type MaxWorkerNumberLimit = MaxWorkerNumberLimit;
    type StakingHandler = DistributionWorkingGroupStakingManager;
    type StakingAccountValidator = Members;
    type MemberOriginValidator = Members;
    type MinUnstakingPeriodLimit = MinUnstakingPeriodLimit;
    type RewardPeriod = DistributionRewardPeriod;
    type WeightInfo = weights::working_group::WeightInfo;
    type MinimumApplicationStake = MinimumApplicationStake;
    type LeaderOpeningStake = LeaderOpeningStake;
}

parameter_types! {
    pub const ProposalCancellationFee: u64 = 10000;
    pub const ProposalRejectionFee: u64 = 5000;
    pub const ProposalTitleMaxLength: u32 = 40;
    pub const ProposalDescriptionMaxLength: u32 = 3000;
    pub const ProposalMaxActiveProposalLimit: u32 = 20;
}

impl proposals_engine::Config for Runtime {
    type Event = Event;
    type ProposerOriginValidator = Members;
    type CouncilOriginValidator = Council;
    type TotalVotersCounter = CouncilManager<Self>;
    type ProposalId = u32;
    type StakingHandler = staking_handler::StakingManager<Self, ProposalsLockId>;
    type CancellationFee = ProposalCancellationFee;
    type RejectionFee = ProposalRejectionFee;
    type TitleMaxLength = ProposalTitleMaxLength;
    type DescriptionMaxLength = ProposalDescriptionMaxLength;
    type MaxActiveProposalLimit = ProposalMaxActiveProposalLimit;
    type DispatchableCallCode = Call;
    type ProposalObserver = ProposalsCodex;
    type WeightInfo = weights::proposals_engine::WeightInfo;
    type StakingAccountValidator = Members;
}

impl Default for Call {
    fn default() -> Self {
        panic!("shouldn't call default for Call");
    }
}

parameter_types! {
    pub const MaxWhiteListSize: u32 = 20;
    pub const ProposalsPostDeposit: Balance = 2000;
    // module : proposals_discussion
    pub const ProposalsDiscussionModuleId: PalletId = PalletId(*b"mo:prdis");
    pub const ForumPostLifeTime: BlockNumber = 3600;
}

macro_rules! call_wg {
    ($working_group:ident, $function:ident $(,$x:expr)*) => {{
        match $working_group {
            WorkingGroup::Content => <ContentWorkingGroup as WorkingGroupBudgetHandler<Runtime>>::$function($($x,)*),
            WorkingGroup::Storage => <StorageWorkingGroup as WorkingGroupBudgetHandler<Runtime>>::$function($($x,)*),
            WorkingGroup::Forum => <ForumWorkingGroup as WorkingGroupBudgetHandler<Runtime>>::$function($($x,)*),
            WorkingGroup::Membership => <MembershipWorkingGroup as WorkingGroupBudgetHandler<Runtime>>::$function($($x,)*),
            WorkingGroup::Gateway => <GatewayWorkingGroup as WorkingGroupBudgetHandler<Runtime>>::$function($($x,)*),
            WorkingGroup::Distribution => <DistributionWorkingGroup as WorkingGroupBudgetHandler<Runtime>>::$function($($x,)*),
            WorkingGroup::OperationsAlpha => <OperationsWorkingGroupAlpha as WorkingGroupBudgetHandler<Runtime>>::$function($($x,)*),
            WorkingGroup::OperationsBeta => <OperationsWorkingGroupBeta as WorkingGroupBudgetHandler<Runtime>>::$function($($x,)*),
            WorkingGroup::OperationsGamma => <OperationsWorkingGroupGamma as WorkingGroupBudgetHandler<Runtime>>::$function($($x,)*),
        }
    }};
}

impl proposals_discussion::Config for Runtime {
    type Event = Event;
    type AuthorOriginValidator = Members;
    type CouncilOriginValidator = Council;
    type ThreadId = ThreadId;
    type PostId = PostId;
    type MaxWhiteListSize = MaxWhiteListSize;
    type WeightInfo = weights::proposals_discussion::WeightInfo;
    type PostDeposit = ProposalsPostDeposit;
    type ModuleId = ProposalsDiscussionModuleId;
    type PostLifeTime = ForumPostLifeTime;
}

impl joystream_utility::Config for Runtime {
    type Event = Event;

    type WeightInfo = weights::joystream_utility::WeightInfo;

    fn get_working_group_budget(working_group: WorkingGroup) -> Balance {
        call_wg!(working_group, get_budget)
    }
    fn set_working_group_budget(working_group: WorkingGroup, budget: Balance) {
        call_wg!(working_group, set_budget, budget)
    }
}

parameter_types! {
    // Make sure to stay below MAX_BLOCK_SIZE of substrate consensus of ~4MB
    pub const RuntimeUpgradeWasmProposalMaxLength: u32 = 3_500_000;
}

impl proposals_codex::Config for Runtime {
    type Event = Event;
    type MembershipOriginValidator = Members;
    type ProposalEncoder = ExtrinsicProposalEncoder;
    type SetMaxValidatorCountProposalParameters = SetMaxValidatorCountProposalParameters;
    type RuntimeUpgradeProposalParameters = RuntimeUpgradeProposalParameters;
    type SignalProposalParameters = SignalProposalParameters;
    type FundingRequestProposalParameters = FundingRequestProposalParameters;
    type CreateWorkingGroupLeadOpeningProposalParameters =
        CreateWorkingGroupLeadOpeningProposalParameters;
    type FillWorkingGroupLeadOpeningProposalParameters =
        FillWorkingGroupLeadOpeningProposalParameters;
    type UpdateWorkingGroupBudgetProposalParameters = UpdateWorkingGroupBudgetProposalParameters;
    type DecreaseWorkingGroupLeadStakeProposalParameters =
        DecreaseWorkingGroupLeadStakeProposalParameters;
    type SlashWorkingGroupLeadProposalParameters = SlashWorkingGroupLeadProposalParameters;
    type SetWorkingGroupLeadRewardProposalParameters = SetWorkingGroupLeadRewardProposalParameters;
    type TerminateWorkingGroupLeadProposalParameters = TerminateWorkingGroupLeadProposalParameters;
    type AmendConstitutionProposalParameters = AmendConstitutionProposalParameters;
    type CancelWorkingGroupLeadOpeningProposalParameters =
        CancelWorkingGroupLeadOpeningProposalParameters;
    type SetMembershipPriceProposalParameters = SetMembershipPriceProposalParameters;
    type SetCouncilBudgetIncrementProposalParameters = SetCouncilBudgetIncrementProposalParameters;
    type SetCouncilorRewardProposalParameters = SetCouncilorRewardProposalParameters;
    type SetInitialInvitationBalanceProposalParameters =
        SetInitialInvitationBalanceProposalParameters;
    type SetInvitationCountProposalParameters = SetInvitationCountProposalParameters;
    type SetMembershipLeadInvitationQuotaProposalParameters =
        SetMembershipLeadInvitationQuotaProposalParameters;
    type SetReferralCutProposalParameters = SetReferralCutProposalParameters;
    type CreateBlogPostProposalParameters = CreateBlogPostProposalParameters;
    type EditBlogPostProoposalParamters = EditBlogPostProoposalParamters;
    type LockBlogPostProposalParameters = LockBlogPostProposalParameters;
    type UnlockBlogPostProposalParameters = UnlockBlogPostProposalParameters;
    type VetoProposalProposalParameters = VetoProposalProposalParameters;
    type WeightInfo = weights::proposals_codex::WeightInfo;
}

impl pallet_constitution::Config for Runtime {
    type Event = Event;
    type WeightInfo = weights::pallet_constitution::WeightInfo;
}

// parameter_types! {
//     pub const BountyModuleId: PalletId = PalletId(*b"m:bounty"); // module : bounty
//     pub const ClosedContractSizeLimit: u32 = 50;
//     pub const MinCherryLimit: Balance = 1000;
//     pub const MinFundingLimit: Balance = 1000;
//     pub const MinWorkEntrantStake: Balance = 1000;
// }

// impl bounty::Config for Runtime {
//     type Event = Event;
//     type ModuleId = BountyModuleId;
//     type BountyId = u64;
//     type Membership = Members;
//     type WeightInfo = weights::bounty::WeightInfo;
//     type CouncilBudgetManager = Council;
//     type StakingHandler = staking_handler::StakingManager<Self, BountyLockId>;
//     type EntryId = u64;
//     type ClosedContractSizeLimit = ClosedContractSizeLimit;
//     type MinCherryLimit = MinCherryLimit;
//     type MinFundingLimit = MinFundingLimit;
//     type MinWorkEntrantStake = MinWorkEntrantStake;
// }

parameter_types! {
    pub const PostsMaxNumber: u64 = 20;
    pub const RepliesMaxNumber: u64 = 100;
    pub const ReplyDeposit: Balance = 2000;
    pub const BlogModuleId: PalletId = PalletId(*b"mod:blog"); // module : forum
    pub const ReplyLifetime: BlockNumber = 43_200;
}

pub type BlogInstance = blog::Instance1;
impl blog::Config<BlogInstance> for Runtime {
    type Event = Event;
    type PostsMaxNumber = PostsMaxNumber;
    type ParticipantEnsureOrigin = Members;
    type WeightInfo = weights::blog::WeightInfo;
    type ReplyId = u64;
    type ReplyDeposit = ReplyDeposit;
    type ModuleId = BlogModuleId;
    type ReplyLifetime = ReplyLifetime;
}

/// Forum identifier for category
pub type CategoryId = u64;

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core datastructures.
pub mod opaque {
    use super::*;

    pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

    /// Opaque block header type.
    pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
    /// Opaque block type.
    pub type Block = generic::Block<Header, UncheckedExtrinsic>;
    /// Opaque block identifier type.
    pub type BlockId = generic::BlockId<Block>;
}

construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = opaque::Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        // Substrate
        System: frame_system,
        Utility: substrate_utility,
        Babe: pallet_babe,
        Timestamp: pallet_timestamp,
        Authorship: pallet_authorship,
        Balances: pallet_balances,
        TransactionPayment: pallet_transaction_payment,
        Staking: pallet_staking,
        Session: pallet_session,
        Historical: pallet_session_historical,
        Grandpa: pallet_grandpa,
        AuthorityDiscovery: pallet_authority_discovery,
        ImOnline: pallet_im_online,
        Offences: pallet_offences,
        RandomnessCollectiveFlip: pallet_randomness_collective_flip,
        Sudo: pallet_sudo,
        // Joystream
        Council: council::{Pallet, Call, Storage, Event<T>, Config<T>},
        Referendum: referendum::<Instance1>::{Pallet, Call, Storage, Event<T>, Config<T>},
        Members: membership::{Pallet, Call, Storage, Event<T>, Config<T>},
        Forum: forum::{Pallet, Call, Storage, Event<T>, Config<T>},
        Constitution: pallet_constitution::{Pallet, Call, Storage, Event},
        // Bounty: bounty::{Pallet, Call, Storage, Event<T>},
        Blog: blog::<Instance1>::{Pallet, Call, Storage, Event<T>},
        JoystreamUtility: joystream_utility::{Pallet, Call, Event<T>},
        Content: content::{Pallet, Call, Storage, Event<T>, Config<T>},
        Storage: storage::{Pallet, Call, Storage, Event<T>},
        // --- Proposals
        ProposalsEngine: proposals_engine::{Pallet, Call, Storage, Event<T>},
        ProposalsDiscussion: proposals_discussion::{Pallet, Call, Storage, Event<T>},
        ProposalsCodex: proposals_codex::{Pallet, Call, Storage, Event<T>},
        // --- Working groups
        ForumWorkingGroup: working_group::<Instance1>::{Pallet, Call, Storage, Event<T>},
        StorageWorkingGroup: working_group::<Instance2>::{Pallet, Call, Storage, Event<T>},
        ContentWorkingGroup: working_group::<Instance3>::{Pallet, Call, Storage, Event<T>},
        OperationsWorkingGroupAlpha: working_group::<Instance4>::{Pallet, Call, Storage, Event<T>},
        GatewayWorkingGroup: working_group::<Instance5>::{Pallet, Call, Storage, Event<T>},
        MembershipWorkingGroup: working_group::<Instance6>::{Pallet, Call, Storage, Event<T>},
        OperationsWorkingGroupBeta: working_group::<Instance7>::{Pallet, Call, Storage, Event<T>},
        OperationsWorkingGroupGamma: working_group::<Instance8>::{Pallet, Call, Storage, Event<T>},
        DistributionWorkingGroup: working_group::<Instance9>::{Pallet, Call, Storage, Event<T>},
    }
);
