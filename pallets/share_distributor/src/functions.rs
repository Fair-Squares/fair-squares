
//1) create VA from nft collection & item Id's --> Done
//2) create tokens
//3) Use onboarding do_buy
//4) transfer tokens to owners
use super::*;
use enum_iterator::all;

impl<T: Config> Pallet<T> {
pub fn virtual_account(collection_id: T::NftCollectionId, item_id: T::NftItemId) -> DispatchResult {
    //Create virtual account
    let text0 = format!("{:?}_{:?}_account",collection_id,item_id.clone());
    let bytes = text0.as_bytes();
    let array: &[u8;8] = &bytes[0..8].try_into().unwrap();
    let account = PalletId(*array).into_account_truncating();

    //Store account inside storage
    Ownership::<T>::new(collection_id.clone(),item_id.clone(),account).ok();
    

    

    Ok(())
}
pub fn nft_transaction(collection_id: T::NftCollectionId, item_id: T::NftItemId,virtual_id:T::AccountId) -> DispatchResult {
    
    //Get collection
        let collection_vec = all::<Nft::PossibleCollections>().collect::<Vec<_>>();
        let _infos = Onboarding::Houses::<T>::get(collection_id.clone(),item_id.clone()).unwrap();
        let mut coll_id = Nft::PossibleCollections::HOUSES;
        for i in collection_vec.iter() {
            let val:T::NftCollectionId=i.value().into();
            if val == collection_id{
                coll_id = *i;                
            }
        }
    //Execute NFT and money transfer
        Onboarding::Pallet::do_buy(coll_id,item_id.clone(),virtual_id.clone(),_infos).ok();        

Ok(())

}

pub fn owner_and_shares(collection_id: T::NftCollectionId, item_id: T::NftItemId) -> Vec<(T::AccountId, f64)>{

    //Get owners and their reserved contribution to the bid
    let infos = HousingFund::Reservations::<T>::get((collection_id,item_id)).unwrap();
    let vec0 = infos.contributions;
    let price = infos.amount;
    let mut vec = Vec::new() ;
    for i in vec0.iter(){
        let float0 = Self::balance_to_float_option(price).unwrap();
        let float1 = Self::balance_to_float_option(i.1.clone()).unwrap();
        let share = float1/float0;
        vec.push((i.0.clone(),share));

    }
    vec
}

pub fn create_tokens(origin: OriginFor<T>, account: T::AccountId) -> DispatchResult{

    //create token class:
    let token_id: <T as pallet::Config>::AssetId = TokenId::<T>::get().into();
    //let token_id: T::AssetId = id.into();
    let to = T::Lookup::unlookup(account.clone());
    Assets::Pallet::<T>::force_create(origin.clone(),token_id.clone().into(),to.clone(),true,Self::u64_to_balance_option(1).unwrap()).ok();
    TokenId::<T>::mutate(|val|{
        let val0 = val.clone();
        *val = val0+1;
    });
      
    //Set class metadata

    //mint 100 tokens
    Assets::Pallet::<T>::mint(origin.clone(),token_id.clone().into(),to,Self::u64_to_balance_option(100).unwrap()).ok();
    

    Ok(())

}

//pub fn distribute_tokens(collection_id: T::NftCollectionId, item_id: T::NftItemId)

// Conversion of u64 to BalanxceOf<T>
pub fn u64_to_balance_option(input: u64) -> Option<T::Balance> {
    input.try_into().ok()
}

// Conversion of BalanceOf<T> to u32
pub fn balance_to_float_option(input: HousingFund::BalanceOf<T>) -> Option<f64> {
    let integer:u64 = input.try_into().ok().unwrap();
    let float = integer as f64;
    Some(float)
}

}