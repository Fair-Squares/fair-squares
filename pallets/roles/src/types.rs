//! # structs
//!
//! Definition and implementation of the different structs found in FairSquares

pub use super::*;
pub use frame_support::{
	assert_ok,
	dispatch::{DispatchResult, EncodeLike},
	inherent::Vec,
	pallet_prelude::*,
	sp_runtime::traits::{AccountIdConversion, Hash, Saturating, StaticLookup, Zero},
	storage::child,
	traits::{
		Currency, ExistenceRequirement, Get, LockableCurrency, ReservableCurrency, WithdrawReasons,
	},
	PalletId,
};
pub use frame_system::{ensure_signed, pallet_prelude::*, RawOrigin};
pub use scale_info::{prelude::vec, TypeInfo};
pub use serde::{Deserialize, Serialize};

pub type BalanceOf<T> =
	<<T as pallet::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub type BlockNumberOf<T> = <T as frame_system::Config>::BlockNumber;
pub type Idle<T> = (Vec<HouseSeller<T>>, Vec<Servicer<T>>);

///This enum contains the roles selectable at account creation
#[derive(Clone, Encode, Decode, PartialEq, Eq, TypeInfo, Copy)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
pub enum Accounts {
	INVESTOR,
	SELLER,
	TENANT,
	SERVICER,
	NOTARY,
	REPRESENTATIVE,
}

impl Default for Accounts {
	fn default() -> Self {
		Accounts::SELLER
	}
}

//-------------------------------------------------------------------------------------
//-------------INVESTOR STRUCT DECLARATION & IMPLEMENTATION_BEGIN----------------------
#[derive(Clone, Encode, Decode, Default, PartialEq, Eq, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Investor<T: Config> {
	pub account_id: T::AccountId,
	pub age: BlockNumberOf<T>,
	pub share: BalanceOf<T>,
	pub selections: u32,
}

impl<T: Config> Investor<T>
where
	types::Investor<T>: EncodeLike<types::Investor<T>>,
{
	//-------------------------------------------------------------------
	//-------------NEW INVESTOR CREATION METHOD_BEGIN--------------------
	pub fn new(acc: OriginFor<T>) -> DispatchResult {
		let caller = ensure_signed(acc)?;
		let now = <frame_system::Pallet<T>>::block_number();

		let inv =
			Investor { account_id: caller.clone(), age: now, share: Zero::zero(), selections: 0 };

		InvestorLog::<T>::insert(caller, &inv);

		Ok(())
	}
	//-------------NEW INVESTOR CREATION METHOD_END--------------------
	//-----------------------------------------------------------------
}
//-------------INVESTOR STRUCT DECLARATION & IMPLEMENTATION_END----------------------
//-----------------------------------------------------------------------------------

//--------------------------------------------------------------------------------------
//-------------HOUSE SELLER STRUCT DECLARATION & IMPLEMENTATION_BEGIN----------------------
#[derive(Clone, Encode, Decode, Default, PartialEq, Eq, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct HouseSeller<T: Config> {
	pub account_id: T::AccountId,
	pub age: BlockNumberOf<T>,
	pub activated: bool,
	pub verifier: T::AccountId,
}
impl<T: Config> HouseSeller<T>
where
	types::HouseSeller<T>: EncodeLike<types::HouseSeller<T>>,
{
	//--------------------------------------------------------------------
	//-------------HOUSE SELLER CREATION METHOD_BEGIN----------------------
	pub fn new(acc: OriginFor<T>) -> DispatchResult {
		let caller = ensure_signed(acc)?;
		let admin = SUDO::Pallet::<T>::key().unwrap();
		let now = <frame_system::Pallet<T>>::block_number();
		ensure!(!HouseSellerLog::<T>::contains_key(&caller), Error::<T>::NoneValue);

		let hw = HouseSeller { account_id: caller, age: now, activated: false, verifier: admin };

		SellerApprovalList::<T>::mutate(|list| {
			list.push(hw);
		});

		Ok(())
	}

	//-------------HOUSE SELLER CREATION METHOD_END----------------------
	//------------------------------------------------------------------
}
//-------------HOUSE SELLER STRUCT DECLARATION & IMPLEMENTATION_END----------------------
//--------------------------------------------------------------------------------------

//--------------------------------------------------------------------------------------
//-------------TENANT STRUCT DECLARATION & IMPLEMENTATION_BEGIN---------------------------
#[derive(Clone, Encode, Decode, Default, PartialEq, Eq, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Tenant<T: Config> {
	pub account_id: T::AccountId,
	pub rent: BalanceOf<T>,
	pub age: BlockNumberOf<T>,
	pub asset_account: Option<T::AccountId>,
	pub contract_start: BlockNumberOf<T>,
	pub remaining_rent: BalanceOf<T>,
	pub remaining_payments:u8,
}
impl<T: Config> Tenant<T> {
	pub fn new(acc: OriginFor<T>) -> DispatchResult {
		let caller = ensure_signed(acc)?;
		let now = <frame_system::Pallet<T>>::block_number();
		let tenant = Tenant {
			account_id: caller.clone(),
			rent: Zero::zero(),
			age: now.clone(),
			asset_account: None,
			contract_start: now,
			remaining_rent: Zero::zero(),
			remaining_payments: 0,
		};
		TenantLog::<T>::insert(caller, &tenant);
		Ok(())
	}
}
//-------------TENANT STRUCT DECLARATION & IMPLEMENTATION_END---------------------------
//--------------------------------------------------------------------------------------

//--------------------------------------------------------------------------------------
//-------------Servicer STRUCT DECLARATION & IMPLEMENTATION_BEGIN---------------------------
#[derive(Clone, Encode, Decode, Default, PartialEq, Eq, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Servicer<T: Config> {
	pub account_id: T::AccountId,
	pub age: BlockNumberOf<T>,
	pub activated: bool,
	pub verifier: T::AccountId,
}
impl<T: Config> Servicer<T> {
	pub fn new(acc: OriginFor<T>) -> DispatchResult {
		let caller = ensure_signed(acc)?;
		let admin = SUDO::Pallet::<T>::key().unwrap();
		let now = <frame_system::Pallet<T>>::block_number();
		let sv = Servicer { account_id: caller, age: now, activated: false, verifier: admin };

		ServicerApprovalList::<T>::mutate(|list| {
			list.push(sv);
		});
		Ok(())
	}
}
//-------------Servicer STRUCT DECLARATION & IMPLEMENTATION_END---------------------------
//--------------------------------------------------------------------------------------

//-------------------------------------------------------------------------------------
//-------------REPRESENTATIVE STRUCT DECLARATION & IMPLEMENTATION_BEGIN----------------------

#[derive(Clone, Encode, Decode, Default, PartialEq, Eq, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Representative<T: Config> {
	pub account_id: T::AccountId,
	pub age: BlockNumberOf<T>,
	pub activated: bool,
	pub assets_accounts: Vec<T::AccountId>,
	pub index: u32
}
impl<T: Config> Representative<T>
where
	types::Representative<T>: EncodeLike<types::Representative<T>>,
{
	//--------------------------------------------------------------------
	//-------------REPRESENTATIVE CREATION METHOD_BEGIN----------------------
	pub fn new(acc: OriginFor<T>) -> DispatchResult {
		let caller = ensure_signed(acc)?;
		let now = <frame_system::Pallet<T>>::block_number();
		ensure!(!RepresentativeLog::<T>::contains_key(&caller), Error::<T>::NoneValue);

		let rep = Representative {
			account_id: caller.clone(),
			age: now,
			activated: false,
			assets_accounts: Vec::new(),
			index: Default::default(),
		};

		RepApprovalList::<T>::mutate(caller, |val| {
			//val.push(rep);
			*val = Some(rep);
		});

		Ok(())
	}

	//-------------HOUSE REPRESENTATIVE CREATION METHOD_END----------------------
	//------------------------------------------------------------------
}

//-------------REPRESENTATIVE STRUCT DECLARATION & IMPLEMENTATION_END----------------------
//-------------------------------------------------------------------------------------

//-------------------------------------------------------------------------------------
//-------------NOTARY STRUCT DECLARATION & IMPLEMENTATION_BEGIN----------------------
#[derive(Clone, Encode, Decode, Default, PartialEq, Eq, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Notary<T: Config> {
	pub account_id: T::AccountId,
	pub age: BlockNumberOf<T>,
	pub activated: bool,
	pub verifier: T::AccountId,
}
impl<T: Config> Notary<T>
where
	types::Notary<T>: EncodeLike<types::Notary<T>>,
{
	pub fn new(acc: OriginFor<T>) -> DispatchResult {
		let caller = ensure_signed(acc)?;
		let now = <frame_system::Pallet<T>>::block_number();

		ensure!(!NotaryLog::<T>::contains_key(&caller), Error::<T>::NoneValue);

		let admin = SUDO::Pallet::<T>::key().unwrap();
		let notary = Notary { account_id: caller, age: now, activated: false, verifier: admin };
		NotaryApprovalList::<T>::mutate(|list| {
			list.push(notary);
		});

		Ok(())
	}
}
//-------------------------------------------------------------------------------------
//-------------NOTARY STRUCT DECLARATION & IMPLEMENTATION_BEGIN----------------------
