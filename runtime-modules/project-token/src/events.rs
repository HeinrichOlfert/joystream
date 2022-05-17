use crate::types::{
    JoyBalanceOf, RevenueSplitId, TokenIssuanceParametersOf, TokenSaleId, TransferPolicyOf,
    ValidatedTransfersOf,
};
use frame_support::decl_event;
use sp_runtime::Perquintill;

decl_event! {
    pub enum Event<T>
    where
        Balance = <T as crate::Trait>::Balance,
        JoyBalance = JoyBalanceOf<T>,
        TokenId = <T as crate::Trait>::TokenId,
        AccountId = <T as frame_system::Trait>::AccountId,
        BlockNumber = <T as frame_system::Trait>::BlockNumber,
        TransferPolicy = TransferPolicyOf<T>,
        TokenIssuanceParameters = TokenIssuanceParametersOf<T>,
        ValidatedTransfers = ValidatedTransfersOf<T>,

    {
        /// Token amount is transferred from src to dst
        /// Params:
        /// - token identifier
        /// - source account
        /// - map containing validated outputs (amount, remark) data indexed by
        ///   (account_id + account existance)
        TokenAmountTransferred(TokenId, AccountId, ValidatedTransfers),

        /// Token amount transferred by issuer
        /// Params:
        /// - token identifier
        /// - source account
        /// - map containing validated outputs
        ///   (amount, opt. vesting schedule, opt. vesting cleanup key, remark) data indexed by
        ///   (account_id + account existance)
        TokenAmountTransferredByIssuer(TokenId, AccountId, ValidatedTransfers),

        /// Patronage rate decreased
        /// Params:
        /// - token identifier
        /// - new patronage rate
        PatronageRateDecreasedTo(TokenId, Perquintill),

        /// Patronage credit claimed by creator
        /// Params:
        /// - token identifier
        /// - credit amount
        /// - account
        PatronageCreditClaimed(TokenId, Balance, AccountId),

        /// Revenue Split issued
        /// Params:
        /// - token identifier
        /// - starting block for the split
        /// - duration of the split
        /// - JOY allocated for the split
        RevenueSplitIssued(TokenId, BlockNumber, BlockNumber, JoyBalance),

        /// Revenue Split finalized
        /// Params:
        /// - token identifier
        /// - recovery account for the leftover funds
        /// - leftover funds
        RevenueSplitFinalized(TokenId, AccountId, JoyBalance),

        /// User partipated in a revenue split
        /// Params:
        /// - token identifier
        /// - user account
        /// - user allocated staked balance
        /// - dividend amount (JOY) granted
        /// - revenue split identifier
        UserParticipatedInSplit(TokenId, AccountId, Balance, JoyBalance, RevenueSplitId),

        /// User left revenue split
        /// Params:
        /// - token identifier
        /// - user account id
        /// - amount unstaked
        RevenueSplitLeft(TokenId, AccountId, Balance),

        /// Member joined whitelist
        /// Params:
        /// - token identifier
        /// - account that has just joined
        /// - ongoing transfer policy
        MemberJoinedWhitelist(TokenId, AccountId, TransferPolicy),

        /// Account Dusted
        /// Params:
        /// - token identifier
        /// - account dusted
        /// - account that called the extrinsic
        /// - ongoing policy
        AccountDustedBy(TokenId, AccountId, AccountId, TransferPolicy),

        /// Token Deissued
        /// Params:
        /// - token id
        TokenDeissued(TokenId),

        /// Token Issued
        /// Params:
        /// - token id
        /// - token issuance parameters
        TokenIssued(TokenId, TokenIssuanceParameters),

        /// Tokens Purchased On Sale
        /// Params:
        /// - token id
        /// - token sale id
        /// - amount of tokens purchased
        /// - address of the buyer
        TokensPurchasedOnSale(TokenId, TokenSaleId, Balance, AccountId),

        /// Unsold Tokens Recovered
        /// Params:
        /// - token id
        /// - token sale id
        /// - amount of tokens recovered
        UnsoldTokensRecovered(TokenId, TokenSaleId, Balance),
    }
}
