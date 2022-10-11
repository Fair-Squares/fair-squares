//! Benchmarking setup for pallet-template

use super::*;

#[allow(unused)]
use crate::Pallet as ShareDistributor;
use frame_benchmarking::{benchmarks, whitelisted_caller,account};
pub use frame_support::{dispatch::{DispatchResult, EncodeLike},sp_runtime::traits::StaticLookup,
inherent::Vec,
pallet_prelude::*};
use frame_system::RawOrigin;
pub type NftColl = Nft::PossibleCollections;
pub type Acc = pallet_roles::Accounts;
const SEED: u32 = 0;


benchmarks! {
	create_virtual{
	
		//let s in 0 .. 100;

		//Create the accounts
		let AA: T::AccountId = Roles::Pallet::<T>::get_manager();
		let BA: T::AccountId =  account("BA",0,SEED);
		let CA: T::AccountId =  account("CA",0,SEED);
		let DA: T::AccountId =  account("DA",0,SEED);
		let EA: T::AccountId =  account("EA",0,SEED);


		//Funding the accounts
		<T as pallet::Config>::Currency::make_free_balance_be(&AA,100_000_000u32.into());
		<T as pallet::Config>::Currency::make_free_balance_be(&BA,100_000_000u32.into());
		<T as pallet::Config>::Currency::make_free_balance_be(&CA,100_000_000u32.into());
		<T as pallet::Config>::Currency::make_free_balance_be(&DA,100_000_000u32.into());
		<T as pallet::Config>::Currency::make_free_balance_be(&EA,100_000_000u32.into());


		//Assign roles to the accounts
		Roles::Pallet::<T>::set_role(RawOrigin::Signed(CA.clone()).clone().into(), CA.clone(), Acc::SERVICER).ok();
		Roles::Pallet::<T>::account_approval(RawOrigin::Signed(AA.clone()).into(), CA.clone()).ok();
		Roles::Pallet::<T>::set_role(RawOrigin::Signed(BA.clone()).clone().into(), BA.clone(), Acc::SELLER).ok();
		Roles::Pallet::<T>::account_approval(RawOrigin::Signed(AA.clone()).into(), BA.clone()).ok();
		Roles::Pallet::<T>::set_role(RawOrigin::Signed(DA.clone()).clone().into(), DA.clone(), Acc::INVESTOR).ok();
		Roles::Pallet::<T>::set_role(RawOrigin::Signed(EA.clone()).clone().into(), EA.clone(), Acc::INVESTOR).ok();

		ensure!(Roles::HouseSellerLog::<T>::contains_key(&BA),"Not good");

		//Investors contribute to HousingFund
		HousingFund::Pallet::<T>::contribute_to_fund(RawOrigin::Signed(DA.clone()).into(),50_000u32.into());
		HousingFund::Pallet::<T>::contribute_to_fund(RawOrigin::Signed(EA.clone()).into(),50_000u32.into());

		//Servicer creates a collection
		let metadata0 = b"metadata0".to_vec().try_into().unwrap();
		let metadata1 = b"metadata1".to_vec().try_into().unwrap();
		Nft::Pallet::<T>::create_collection(RawOrigin::Signed(CA.clone()).into(),Nft::PossibleCollections::APPARTMENTS,metadata0).ok();
		
		let fees_account = Onboarding::Pallet::<T>::account_id();
		<T as pallet::Config>::Currency::make_free_balance_be(&fees_account,150_000u32.into());


		//let caller: T::AccountId = whitelisted_caller();
		let coll_id:<T as Nft::Config>::NftCollectionId = NftColl::APPARTMENTS.value().into();
		let item_idx =  NftColl::APPARTMENTS.value();
		let price = 40_000u32;

		//Seller creates a proposal 
		let res0 = Onboarding::Pallet::<T>::create_and_submit_proposal(
			RawOrigin::Signed(BA.clone()).into(),
			NftColl::APPARTMENTS,
			Some(price.clone().into()),
			metadata1,
			false
		);
		debug_assert!(res0.is_ok());
		

		let item_id: <T as Nft::Config>::NftItemId = (Nft::ItemsCount::<T>::get()[item_idx.clone() as usize] - 1).into();

		let contribution_eve = HousingFund::Contribution {
			account_id: EA.clone(),
			available_balance: HousingFund::Pallet::<T>::u64_to_balance_option(10_000).unwrap(),
			reserved_balance: HousingFund::Pallet::<T>::u64_to_balance_option(25_000).unwrap(),
			contributed_balance: HousingFund::Pallet::<T>::u64_to_balance_option(0).unwrap(),
			has_withdrawn: false,
			block_number: 1u32.into(),
			contributions: vec![HousingFund::ContributionLog {
				amount: HousingFund::Pallet::<T>::u64_to_balance_option(35_000).unwrap(),
				block_number: 1u32.into()
			}],
			withdraws: Vec::new()
		};

		let contribution_dave = HousingFund::Contribution {
			account_id: DA.clone(),
			available_balance: HousingFund::Pallet::<T>::u64_to_balance_option(10_000).unwrap(),
			reserved_balance: HousingFund::Pallet::<T>::u64_to_balance_option(15_000).unwrap(),
			contributed_balance: HousingFund::Pallet::<T>::u64_to_balance_option(0).unwrap(),
			has_withdrawn: false,
			block_number: 1u32.into(),
			contributions: vec![HousingFund::ContributionLog {
				amount: HousingFund::Pallet::<T>::u64_to_balance_option(25_000).unwrap(),
				block_number: 1u32.into()
			}],
			withdraws: Vec::new()
		};

		// Add contributions to storage
		HousingFund::Contributions::<T>::insert(EA.clone(),contribution_eve);
		HousingFund::Contributions::<T>::insert(DA.clone(),contribution_dave);


		//Create a FundOperation struct for this asset
		let fund_op = HousingFund::FundOperation::<T>{
			nft_collection_id: coll_id.clone(),
			nft_item_id: item_id.clone(),
			amount: price.clone().into(),
			block_number:1u32.into(),
			contributions:vec![(EA.clone(),25_000u32.into()),(DA.clone(),15_000u32.into())],
		};

		//Add new owners and asset to housing fund
		HousingFund::Reservations::<T>::insert((coll_id.clone(),item_id.clone()),fund_op);

		// Update the Housing fund to fit with the contributions
		HousingFund::FundBalance::<T>::mutate(|val| {
			*val = HousingFund::FundInfo {
				total: 60_000u32.into(),
				transferable: 20_000u32.into(),
				reserved: 40_000u32.into(),
			};
		});
		let origin2 = RawOrigin::Signed(BA.clone());
		

		//Change first asset status to FINALISED
		Onboarding::Pallet::<T>::change_status(origin2.into(),NftColl::APPARTMENTS,item_id.clone(),Onboarding::AssetStatus::FINALISED).ok();




		
	}: _(RawOrigin::Root,coll_id.clone(),item_id.clone())
	verify {
		
        assert_eq!(ShareDistributor::<T>::virtual_acc(coll_id.clone(),item_id.clone()).unwrap().owners.len()>1,true);
    }

	impl_benchmark_test_suite!(ShareDistributor, mock::ExtBuilder::default().build(), crate::mock::Test);
}

