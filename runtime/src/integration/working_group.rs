use frame_support::StorageMap;
use sp_std::marker::PhantomData;

use crate::{
    ContentWorkingGroupInstance, GatewayWorkingGroupInstance, OperationsWorkingGroupInstance,
    StorageWorkingGroupInstance,
};
use stake::{BalanceOf, NegativeImbalance};

// Will be removed in the next releases.
#[allow(clippy::upper_case_acronyms)]
pub struct ContentDirectoryWgStakingEventsHandler<T> {
    pub marker: PhantomData<T>,
}

impl<T: stake::Trait + working_group::Trait<ContentWorkingGroupInstance>>
    stake::StakingEventsHandler<T> for ContentDirectoryWgStakingEventsHandler<T>
{
    /// Unstake remaining sum back to the source_account_id
    fn unstaked(
        stake_id: &<T as stake::Trait>::StakeId,
        _unstaked_amount: BalanceOf<T>,
        remaining_imbalance: NegativeImbalance<T>,
    ) -> NegativeImbalance<T> {
        // Stake not related to a staked role managed by the hiring module.
        if !hiring::ApplicationIdByStakingId::<T>::contains_key(*stake_id) {
            return remaining_imbalance;
        }

        let hiring_application_id = hiring::ApplicationIdByStakingId::<T>::get(*stake_id);

        if working_group::MemberIdByHiringApplicationId::<T, ContentWorkingGroupInstance>::contains_key(
            hiring_application_id,
        ) {
            return <working_group::Module<T, ContentWorkingGroupInstance>>::refund_working_group_stake(
				*stake_id,
				remaining_imbalance,
			);
        }

        remaining_imbalance
    }

    /// Empty handler for the slashing.
    fn slashed(
        _: &<T as stake::Trait>::StakeId,
        _: Option<<T as stake::Trait>::SlashId>,
        _: BalanceOf<T>,
        _: BalanceOf<T>,
        remaining_imbalance: NegativeImbalance<T>,
    ) -> NegativeImbalance<T> {
        remaining_imbalance
    }
}

pub struct StorageWgStakingEventsHandler<T> {
    pub marker: PhantomData<T>,
}

impl<T: stake::Trait + working_group::Trait<StorageWorkingGroupInstance>>
    stake::StakingEventsHandler<T> for StorageWgStakingEventsHandler<T>
{
    /// Unstake remaining sum back to the source_account_id
    fn unstaked(
        stake_id: &<T as stake::Trait>::StakeId,
        _unstaked_amount: BalanceOf<T>,
        remaining_imbalance: NegativeImbalance<T>,
    ) -> NegativeImbalance<T> {
        // Stake not related to a staked role managed by the hiring module.
        if !hiring::ApplicationIdByStakingId::<T>::contains_key(*stake_id) {
            return remaining_imbalance;
        }

        let hiring_application_id = hiring::ApplicationIdByStakingId::<T>::get(*stake_id);

        if working_group::MemberIdByHiringApplicationId::<T, StorageWorkingGroupInstance>::contains_key(
            hiring_application_id,
        ) {
            return <working_group::Module<T, StorageWorkingGroupInstance>>::refund_working_group_stake(
				*stake_id,
				remaining_imbalance,
			);
        }

        remaining_imbalance
    }

    /// Empty handler for the slashing.
    fn slashed(
        _: &<T as stake::Trait>::StakeId,
        _: Option<<T as stake::Trait>::SlashId>,
        _: BalanceOf<T>,
        _: BalanceOf<T>,
        remaining_imbalance: NegativeImbalance<T>,
    ) -> NegativeImbalance<T> {
        remaining_imbalance
    }
}

pub struct OperationsWgStakingEventsHandler<T> {
    pub marker: PhantomData<T>,
}

impl<T: stake::Trait + working_group::Trait<OperationsWorkingGroupInstance>>
    stake::StakingEventsHandler<T> for OperationsWgStakingEventsHandler<T>
{
    /// Unstake remaining sum back to the source_account_id
    fn unstaked(
        stake_id: &<T as stake::Trait>::StakeId,
        _unstaked_amount: BalanceOf<T>,
        remaining_imbalance: NegativeImbalance<T>,
    ) -> NegativeImbalance<T> {
        // Stake not related to a staked role managed by the hiring module.
        if !hiring::ApplicationIdByStakingId::<T>::contains_key(*stake_id) {
            return remaining_imbalance;
        }

        let hiring_application_id = hiring::ApplicationIdByStakingId::<T>::get(*stake_id);

        if working_group::MemberIdByHiringApplicationId::<T, OperationsWorkingGroupInstance>::contains_key(
            hiring_application_id,
        ) {
            return <working_group::Module<T, OperationsWorkingGroupInstance>>::refund_working_group_stake(
				*stake_id,
				remaining_imbalance,
			);
        }

        remaining_imbalance
    }

    /// Empty handler for the slashing.
    fn slashed(
        _: &<T as stake::Trait>::StakeId,
        _: Option<<T as stake::Trait>::SlashId>,
        _: BalanceOf<T>,
        _: BalanceOf<T>,
        remaining_imbalance: NegativeImbalance<T>,
    ) -> NegativeImbalance<T> {
        remaining_imbalance
    }
}

pub struct GatewayWgStakingEventsHandler<T> {
    pub marker: PhantomData<T>,
}

impl<T: stake::Trait + working_group::Trait<GatewayWorkingGroupInstance>>
    stake::StakingEventsHandler<T> for GatewayWgStakingEventsHandler<T>
{
    /// Unstake remaining sum back to the source_account_id
    fn unstaked(
        stake_id: &<T as stake::Trait>::StakeId,
        _unstaked_amount: BalanceOf<T>,
        remaining_imbalance: NegativeImbalance<T>,
    ) -> NegativeImbalance<T> {
        // Stake not related to a staked role managed by the hiring module.
        if !hiring::ApplicationIdByStakingId::<T>::contains_key(*stake_id) {
            return remaining_imbalance;
        }

        let hiring_application_id = hiring::ApplicationIdByStakingId::<T>::get(*stake_id);

        if working_group::MemberIdByHiringApplicationId::<T, GatewayWorkingGroupInstance>::contains_key(
            hiring_application_id,
        ) {
            return <working_group::Module<T, GatewayWorkingGroupInstance>>::refund_working_group_stake(
				*stake_id,
				remaining_imbalance,
			);
        }

        remaining_imbalance
    }

    /// Empty handler for the slashing.
    fn slashed(
        _: &<T as stake::Trait>::StakeId,
        _: Option<<T as stake::Trait>::SlashId>,
        _: BalanceOf<T>,
        _: BalanceOf<T>,
        remaining_imbalance: NegativeImbalance<T>,
    ) -> NegativeImbalance<T> {
        remaining_imbalance
    }
}
