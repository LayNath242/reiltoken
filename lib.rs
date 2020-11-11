#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod owner {
    #[ink(storage)]
    pub struct Owner {
        owner: AccountId,
    }

    impl Owner {
        #[ink(constructor)]
        pub fn new(owner: AccountId) -> Self {
            Self { owner }
        }

        #[ink(message)]
        pub fn transfer_ownership(&mut self, to: AccountId) -> bool {
            if self.only_owner(self.owner) {
                self.owner = to;
                true
            } else {
                false
            }
        }

        #[ink(message)]
        pub fn only_owner(&self, caller: AccountId) -> bool {
            if self.owner == caller {
                true
            } else {
                false
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn new_works() {
            Owner::new(AccountId::from([0x1; 32]));
        }
        #[test]
        fn is_owner_works() {
            let contract = Owner::new(AccountId::from([0x1; 32]));
            assert_eq!(contract.only_owner(AccountId::from([0x0; 32])), false);
        }

        #[test]
        fn chage_owner_works() {
            let mut contract = Owner::new(AccountId::from([0x1; 32]));
            contract.transfer_ownership(AccountId::from([0x0; 32]));
            assert_eq!(contract.only_owner(AccountId::from([0x0; 32])), true);
        }
    }
}
