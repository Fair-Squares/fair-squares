pub use super::*;
pub use frame_support::{
	assert_ok,
	dispatch::{DispatchResult},
	pallet_prelude::*,
	sp_runtime::traits::{AccountIdConversion, Hash, Saturating, StaticLookup, Zero},
	storage::{child,bounded_vec::BoundedVec},
	traits::{
		UnfilteredDispatchable,Currency, ExistenceRequirement, Get, LockableCurrency, ReservableCurrency, WithdrawReasons,
	},
	dispatch::GetDispatchInfo,
	PalletId,
};
pub use sp_std::vec::Vec;
pub use frame_system::{ensure_signed, ensure_root, pallet_prelude::*, RawOrigin};
pub use scale_info::{prelude::vec, TypeInfo};
pub use serde::{Deserialize, Serialize};

pub type BalanceOf<T> =
	<<T as Roles::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub type BlockNumberOf<T> = BlockNumberFor<T>;
pub type EligibleContribution<T> = (AccountIdOf<T>, BalanceOf<T>,BalanceOf<T>);
pub type UserBalance<T> = (AccountIdOf<T>, BalanceOf<T>);

#[derive(Clone,Encode, Decode, Default, RuntimeDebug,PartialEq, Eq, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
//#[cfg_attr(feature = "std", derive(Debug))]
pub struct InvestmentRound<T: Config>{
	pub round_number:u32,
	pub investors: BoundedVec<UserBalance<T> ,<T as Houses::Config>::MaxInvestorPerHouse>,
	pub collection_id: T::NftCollectionId,
	pub item_id: T::NftItemId,
	pub when: BlockNumberOf<T>,
}
impl<T: Config>InvestmentRound<T>{
	pub fn new(collection_id: T::NftCollectionId, item_id: T::NftItemId) -> Self{
		let now = <frame_system::Pallet<T>>::block_number();
		let round = InvestmentRoundCount::<T>::get().unwrap();

		if !InvestorsList::<T>::contains_key(collection_id,item_id){
			let list = InvestmentRound::<T>{
				round_number: round,
				investors: BoundedVec::new(),
				collection_id,
				item_id,
				when: now
			};
			InvestorsList::<T>::insert(collection_id,item_id,list.clone());
			list
		} else {
			let list = InvestorsList::<T>::get(collection_id,item_id).unwrap();
			list
		}
		
		

	}
}