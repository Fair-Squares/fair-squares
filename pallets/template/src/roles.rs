
pub struct Investor<T,U>{
    account_id:T,
    nft:U,
}
impl<T,U> Investor<T,U>{
    pub fn new(acc: T,nft:U)-> Self{
        Investor{
            account_id: acc,
            nft: nft,
        }
        
    }
}

pub struct HouseOwner<T,U>{
    account_id:T,
    nft:U,
}
pub struct Tenant<T,U>{
    account_id:T,
    rent:U,
}
