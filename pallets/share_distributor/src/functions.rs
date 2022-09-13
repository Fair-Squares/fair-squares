
//1) create VA from nft collection & item Id's --> Done
//2) create tokens
//3) Use onboarding do_buy
//4) transfer tokens to owners
use super::*;

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
}