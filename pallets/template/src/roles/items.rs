pub use super::*;


pub struct House<T,U,V,W,X>{
    pub houseowner:T,
    pub valuation:U,
    pub rent:U,
    pub balance:U,
    pub class_id:V,
    pub token_id:W,
    pub funded:X,    
}



pub struct Contribution<T,U>{
    account:T,
    amount:U,
}
impl<T,U>Contribution<T,U>{
    pub fn new(acc:T,val:U)-> Self{
        Self{
            account:acc,
            amount:val,
        }
    }
}