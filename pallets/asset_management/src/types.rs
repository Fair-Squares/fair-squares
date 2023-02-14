pub use super::*;
pub use frame_support::{
	assert_ok,
	dispatch::{DispatchResult, EncodeLike},
	inherent::Vec,
	pallet_prelude::*,
	sp_runtime::{
		traits::{AccountIdConversion, Hash, One, Saturating, StaticLookup, Zero},
		FixedU128, PerThing, Percent,
	},
	storage::child,
	traits::{
		Contains, Currency, ExistenceRequirement, Get, LockableCurrency, ReservableCurrency,
		UnfilteredDispatchable, WithdrawReasons,
	},
	weights::GetDispatchInfo,
	PalletId,
};
pub use frame_system::{ensure_signed, pallet_prelude::*, RawOrigin};
pub use scale_info::{prelude::vec, TypeInfo};
pub use sp_runtime::{
	traits::{BadOrigin, BlakeTwo256, IdentityLookup},
	Perbill,
};
pub use sp_std::boxed::Box;
pub use Payment::PaymentHandler;

pub type BalanceOf<T> =
	<<T as pallet::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub type BlockNumberOf<T> = <T as frame_system::Config>::BlockNumber;
pub type HashOf<T> = <T as frame_system::Config>::Hash;

pub type DemoBalanceOf<T> =
	<<T as Dem::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

pub type IdentBalanceOf<T> =
	<<T as Ident::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

pub type AssetsBalanceOf<T> =
	<<T as Assetss::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

pub type RefInfos<T> =
	pallet_democracy::ReferendumInfo<BlockNumberOf<T>, HashOf<T>, DemoBalanceOf<T>>;

#[derive(Clone, Encode, Decode, PartialEq, Eq, TypeInfo, Copy)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum VoteResult {
	AWAITING,
	ACCEPTED,
	REJECTED,
}

#[derive(Clone, Encode, Decode, PartialEq, Eq, TypeInfo, Copy)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum BalanceTypes {
	Assets,
	Dem,
	HFund,
	Ident,
	Nft,
	Onboarding,
	Payment,
	Roles,
	Share,
	Manage,
}

#[derive(Clone, Encode, Decode, PartialEq, Eq, TypeInfo, Copy)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum VoteProposals {
	Election,
	Demotion,
}

#[derive(Clone, Encode, Decode, PartialEq, Eq, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct ProposalRecord<T: Config> {
	///Asset owner who made the proposal
	pub caller_account: T::AccountId,
	///Virtual account corresponding to the asset
	pub virtual_account: T::AccountId,
	///Asset collection_id
	pub collection_id: T::NftCollectionId,
	///Asset item_id
	pub item_id: T::NftItemId,
	///Candidate for the representative role
	pub candidate_account: T::AccountId,
	///Vote result
	pub vote_result: VoteResult,
	///Session creation block
	pub when: BlockNumberOf<T>,
}

impl<T: Config> ProposalRecord<T> {
	pub fn new(
		caller_account: T::AccountId,
		virtual_account: T::AccountId,
		candidate_account: T::AccountId,
		referendum_index: Dem::ReferendumIndex,
		collection_id: T::NftCollectionId,
		item_id: T::NftItemId,
	) -> DispatchResult {
		let vote_result = VoteResult::AWAITING;
		let when = <frame_system::Pallet<T>>::block_number();
		let session = ProposalRecord::<T> {
			caller_account: caller_account.clone(),
			virtual_account,
			collection_id,
			item_id,
			candidate_account,
			vote_result,
			when,
		};
		ProposalsLog::<T>::insert(referendum_index, session);
		ProposalsIndexes::<T>::insert(caller_account, referendum_index);
		Ok(())
	}
}

#[derive(Clone, Encode, Decode, PartialEq, Eq, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct BalanceType<T: Config> {
	pub hfund_bal: HFund::BalanceOf<T>,
	pub roles_bal: Roles::BalanceOf<T>,
	pub onboarding_bal: Onboarding::BalanceOf<T>,
	pub share_bal: Share::BalanceOf<T>,
	pub dem_bal: DemoBalanceOf<T>,
	pub assets_bal: AssetsBalanceOf<T>,
	pub ident_bal: IdentBalanceOf<T>,
	pub payment_bal: Payment::BalanceOf<T>,
	pub manage_bal: BalanceOf<T>,
}

impl<T: Config> BalanceType<T> {
	pub fn convert_to_balance(number: u128) -> Self {
		let roles_bal = Zero::zero();
		let hfund_bal = Zero::zero();
		let onboarding_bal = Zero::zero();
		let share_bal = Zero::zero();
		let dem_bal = Zero::zero();
		let assets_bal = Zero::zero();
		let ident_bal = Zero::zero();
		let payment_bal = Zero::zero();
		let manage_bal = Zero::zero();
		let mut new = BalanceType::<T> {
			roles_bal,
			hfund_bal,
			onboarding_bal,
			share_bal,
			dem_bal,
			assets_bal,
			ident_bal,
			payment_bal,
			manage_bal,
		};
		new.hfund_bal = number.try_into().ok().unwrap();
		new.roles_bal = number.try_into().ok().unwrap();
		new.onboarding_bal = number.try_into().ok().unwrap();
		new.share_bal = number.try_into().ok().unwrap();
		new.dem_bal = number.try_into().ok().unwrap();
		new.assets_bal = number.try_into().ok().unwrap();
		new.ident_bal = number.try_into().ok().unwrap();
		new.payment_bal = number.try_into().ok().unwrap();
		new.manage_bal = number.try_into().ok().unwrap();

		new
	}
}
