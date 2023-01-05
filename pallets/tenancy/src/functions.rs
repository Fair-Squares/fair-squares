use super::*;
#[allow(unused_imports)]

impl <T: Config> Pallet<T> {

    pub fn request_asset(
        origin: OriginFor<T>,
        info: Box<IdentityInfo<T::MaxAdditionalFields>>,
        asset_type: Nft::PossibleCollections,
        asset_id: u32,
    ) -> DispatchResult{
        Ok(())
    }
}