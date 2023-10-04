use super::*;

impl<T: Config> Pallet<T> {
	pub fn create_asset(
		origin: OriginFor<T>,
		collection: NftCollectionOf,
		metadata: Nft::BoundedVecOfNfts<T>,
		new_price: Option<BalanceOf<T>>,
		item_id: T::NftItemId,
		max_tenants: u8,
	) -> DispatchResult {
		let coll_id: T::NftCollectionId = collection.clone().value().into();
		// Mint nft
		Nft::Pallet::<T>::mint(origin.clone(), collection.into(), metadata.into()).ok();

		let infos = Nft::Items::<T>::get(coll_id, item_id).unwrap();
		// Set asset price
		Self::price(origin, collection, item_id, new_price).ok();
		// Create Asset
		let asset=Asset::<T>::new(coll_id, item_id, infos, new_price,max_tenants);
		let owner = pallet_nft::Pallet::<T>::owner(coll_id,item_id).unwrap();
		pallet_roles::Asset::<T>::insert(owner,asset.created,asset.status);

		Ok(())
	}

	pub fn status(collection: NftCollectionOf, item_id: T::NftItemId, status: pallet_roles::AssetStatus) {
		let collection_id: T::NftCollectionId = collection.clone().value().into();
		let owner = pallet_nft::Pallet::<T>::owner(collection_id,item_id).unwrap();

		
		Houses::<T>::mutate(collection_id, item_id, |val| {
			let mut asset = val.clone().unwrap();
			asset.status = status.clone();
			*val = Some(asset);
		});
		let asset = Self::houses(collection_id, item_id).unwrap();
		pallet_roles::Asset::<T>::mutate(owner,asset.created,|val|{
			*val=Some(status);
		})

	}

	pub fn price(
		origin: OriginFor<T>,
		collection: NftCollectionOf,
		item_id: T::NftItemId,
		new_price: Option<BalanceOf<T>>,
	) -> DispatchResult {
		let sender = ensure_signed(origin)?;
		let collection_id: T::NftCollectionId = collection.clone().value().into();

		ensure!(
			pallet_nft::Pallet::<T>::owner(collection_id, item_id) == Some(sender.clone()),
			Error::<T>::NotTheTokenOwner
		);
		Prices::<T>::mutate_exists(collection_id, item_id, |price| *price = new_price);

		Self::deposit_event(Event::TokenPriceUpdated {
			who: sender,
			collection: collection_id,
			item: item_id,
			price: new_price,
		});

		Ok(())
	}

	///Execute the buy/sell transaction

	pub fn do_buy(
		collection: NftCollectionOf,
		item_id: T::NftItemId,
		buyer: T::AccountId,
		_infos: Asset<T>,
	) -> DispatchResult {
		let collection_id: T::NftCollectionId = collection.clone().value().into();
		let origin_root: OriginFor<T> = frame_system::RawOrigin::Root.into();
		let origin_buyer: OriginFor<T> = frame_system::RawOrigin::Signed(buyer.clone()).into();

		//Check that the house item exists and has the correct status
		ensure!(
			Houses::<T>::contains_key(collection_id, item_id),
			Error::<T>::CollectionOrItemUnknown
		);
		let asset = Self::houses(collection_id, item_id).unwrap();
		let status = asset.status;
		ensure!(status == pallet_roles::AssetStatus::FINALISED, Error::<T>::VoteNedeed);

		//Check that the owner is not the buyer
		let owner = Nft::Pallet::<T>::owner(collection_id, item_id)
			.ok_or(Error::<T>::CollectionOrItemUnknown)?;
		ensure!(buyer != owner, Error::<T>::BuyFromSelf);
		let balance = <T as Config>::Currency::reserved_balance(&owner);
		let _returned = <T as Config>::Currency::unreserve(&owner, balance);

		// The reserved funds in Housing Fund from the house bidding are unreserved for the transfer
		// transaction
		HousingFund::Pallet::<T>::unreserve_house_bidding_amount(collection_id, item_id).ok();

		//Transfer funds from HousingFund to owner
		let price = Prices::<T>::get(collection_id, item_id).unwrap();
		let fund_id = T::PalletId::get().into_account_truncating();
		<T as Config>::Currency::transfer(
			&fund_id,
			&owner,
			price,
			ExistenceRequirement::KeepAlive,
		)?;
		let to = T::Lookup::unlookup(buyer.clone());
		Nft::Pallet::<T>::transfer(origin_root, collection, item_id, to)?;
		Self::deposit_event(Event::TokenSold {
			owner,
			buyer,
			collection: collection_id,
			item: item_id,
			price,
		});

		//change status
		Self::change_status(origin_buyer, collection, item_id, pallet_roles::AssetStatus::PURCHASED).ok();

		Ok(())
	}

	pub fn make_proposal(call: CallOf<T>) -> BoundedCallOf<T> {
		<T as DEM::Config>::Preimages::bound(call).unwrap()
	}

	pub fn start_dem_referendum(proposal:BoundedCallOf<T> ,delay:BlockNumberFor<T>) -> DEM::ReferendumIndex{
		let threshold = DEM::VoteThreshold::SimpleMajority;    
		let referendum_index =
				DEM::Pallet::<T>::internal_start_referendum(proposal, threshold, delay);
		referendum_index
	}


	pub fn account_id() -> T::AccountId {
		T::FeesAccount::get().into_account_truncating()
	}

	pub fn account_vote(b: BalanceOf1<T>, choice:bool) -> DEM::AccountVote<BalanceOf1<T>> {
		let v = DEM::Vote { aye: choice, conviction: DEM::Conviction::Locked1x };
	
		DEM::AccountVote::Standard { vote: v, balance: b }
	}
	
	pub fn get_formatted_call(call: Call<T>) -> <T as Config>::Prop {
		call.into()
	}

	fn get_houses_by_status(
		status: pallet_roles::AssetStatus,
	) -> Vec<(
		<T as pallet_nft::Config>::NftCollectionId,
		<T as pallet_nft::Config>::NftItemId,
		types::Asset<T>,
	)> {
		Houses::<T>::iter()
			.filter(|(_, _, house)| house.status == status)
			.map(|(collection_id, item_id, house)| (collection_id, item_id, house))
			.collect()
	}

	pub fn get_onboarded_houses() -> Vec<(
		<T as pallet_nft::Config>::NftCollectionId,
		<T as pallet_nft::Config>::NftItemId,
		types::Asset<T>,
	)> {
		Self::get_houses_by_status(pallet_roles::AssetStatus::ONBOARDED)
	}

	pub fn get_finalised_houses() -> Vec<(
		<T as pallet_nft::Config>::NftCollectionId,
		<T as pallet_nft::Config>::NftItemId,
		types::Asset<T>,
	)> {
		Self::get_houses_by_status(pallet_roles::AssetStatus::FINALISED)
	}

	pub fn get_finalising_houses() -> Vec<(
		<T as pallet_nft::Config>::NftCollectionId,
		<T as pallet_nft::Config>::NftItemId,
		types::Asset<T>,
	)> {
		Self::get_houses_by_status(pallet_roles::AssetStatus::FINALISING)
	}

	pub fn do_submit_proposal(
		collection: NftCollectionOf,
		item_id: T::NftItemId,
	) {
		//Change asset status to REVIEWING
		Self::change_status(frame_system::RawOrigin::Root.into(), collection, item_id, pallet_roles::AssetStatus::REVIEWING).ok();
		//Send Proposal struct to voting pallet
		//get the needed call and convert them to pallet_voting format
		let collection_id: T::NftCollectionId = collection.clone().value().into();
		let out_call = Vcalls::<T>::get(collection_id, item_id).unwrap();
		let call0 = Self::get_formatted_call(out_call.after_vote_status) ;

		
		let proposal = Self::make_proposal(call0.into());
		let delay = T::Delay::get();
			let index=Self::start_dem_referendum(proposal,delay);
		Houses::<T>::mutate_exists(collection_id,item_id,|val|{
			let mut v0 = val.clone().unwrap();
			v0.ref_index=index;
			*val = Some(v0);
		});
		
	}

	pub fn collection_name(index:u32)-> Nft::PossibleCollections{

		match index {
			0 => Nft::PossibleCollections::HOUSES,
			1 => Nft::PossibleCollections::OFFICES,
			2 => Nft::PossibleCollections::APPARTMENTS,
			3 => Nft::PossibleCollections::HOUSESTEST,
			4 => Nft::PossibleCollections::OFFICESTEST,
			5 => Nft::PossibleCollections::APPARTMENTSTEST,
			6 => Nft::PossibleCollections::NONEXISTING,
			_ => Nft::PossibleCollections::NONEXISTING,
		}
	}

	pub fn begin_block(now: BlockNumberOf<T>) -> Weight{
		let max_block_weight = Weight::from_parts(1000_u64,0);
		if(now % T::CheckDelay::get()).is_zero(){
			//get existing assets
			let assets_iter = Houses::<T>::iter();
			for asset in assets_iter{
				let coll_id = asset.0;
				let item_id = asset.1;
				let status = asset.2.status;
				let index = asset.2.ref_index;

				//Use index to get referendum infos
				let infos = DEM::Pallet::<T>::referendum_info(index).unwrap();
				let b = match infos {
					pallet_democracy::ReferendumInfo::Finished { approved, end: _ } => {
						(1, approved)
					},
					_ => (0, false),
				};
				if b.0 == 1{
					let coll = Self::collection_name(coll_id.into());
					//Prepare & execute rejection call
					let call2: T::Prop =
				Call::<T>::reject_edit { collection:coll, item_id, infos: asset.2.clone() }.into();
				call2.dispatch_bypass_filter(frame_system::RawOrigin::Root.into()).ok();
				}

			}


		}
		
		max_block_weight
	}

	
}