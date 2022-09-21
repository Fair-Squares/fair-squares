
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

}