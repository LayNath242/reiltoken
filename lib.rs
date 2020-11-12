#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod riel_token {
    use ink_storage::{collections::HashMap as StorageHashMap, lazy::Lazy};

    #[derive(Debug, PartialEq, Eq, scale::Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        InsufficientBalance,
        InsufficientAllowance,
        OnlyOwner,
        EvilAccount,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    /// ERC-20 contract for RielToken.
    #[cfg(not(feature = "ink-as-dependency"))]
    #[ink(storage)]
    pub struct RielToken {
        ///Owner of Contract.
        owner: Lazy<AccountId>,
        /// Total token supply.
        total_supply: Lazy<Balance>,
        /// Mapping from owner to number of owned token.
        balances: StorageHashMap<AccountId, Balance>,
        /// Mapping of the token amount which an account is allowed to withdraw from another account.
        allowances: StorageHashMap<(AccountId, AccountId), Balance>,
        /// Mapping of the blacklist account
        blacklist: StorageHashMap<AccountId, bool>,
    }

    /// Event emitted when a token transfer occurs.
    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        #[ink(topic)]
        value: Balance,
    }

    ///Event emitted when ownership have transfer
    #[ink(event)]
    pub struct TransferOwnerShip {
        #[ink(topic)]
        from: AccountId,
        #[ink(topic)]
        to: AccountId,
    }

    /// Event emitted when an approval occurs that `spender` is allowed to withdraw
    /// up to the amount of `value` tokens from `owner`.
    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        spender: AccountId,
        #[ink(topic)]
        value: Balance,
    }

    ///Event emit when total have increment
    #[ink(event)]
    pub struct IncrementSupply {
        #[ink(topic)]
        from: AccountId,
        #[ink(topic)]
        value: Balance,
    }

    ///Event emit when total have decrement
    #[ink(event)]
    pub struct DecrementSupply {
        #[ink(topic)]
        from: AccountId,
        #[ink(topic)]
        value: Balance,
    }

    impl RielToken {
        /// Creates a new Reil contract with the initial supply and owner.
        #[ink(constructor)]
        pub fn new(initial_supply: Balance) -> Self {
            let caller = Self::env().caller();
            let mut balances = StorageHashMap::new();
            balances.insert(caller, initial_supply);

            Self::env().emit_event(Transfer {
                from: None,
                to: Some(caller),
                value: initial_supply,
            });

            Self {
                owner: Lazy::new(caller),
                total_supply: Lazy::new(initial_supply),
                balances,
                allowances: StorageHashMap::new(),
                blacklist: StorageHashMap::new(),
            }
        }

        /// Returns the total token supply.
        #[ink(message)]
        pub fn total_supply(&self) -> Balance {
            *self.total_supply
        }

        /// Change ownership only by owner.
        #[ink(message)]
        pub fn transfer_ownership(&mut self, to: AccountId) -> Result<()> {
            let owner = *self.owner;
            self.only_owner(*self.owner)?;
            *self.owner = to;
            self.env().emit_event(TransferOwnerShip { from: owner, to });
            Ok(())
        }

        #[ink(message)]
        pub fn add_blacklist(&mut self, evil_user: AccountId) -> Result<()> {
            let caller = Self::env().caller();
            self.only_owner(caller)?;
            self.blacklist.insert(evil_user, true);

            Ok(())
        }

        #[ink(message)]
        pub fn remove_blacklist(&mut self, evil_user: AccountId) -> Result<()> {
            let caller = Self::env().caller();
            self.only_owner(caller)?;
            self.blacklist.insert(evil_user, false);
            Ok(())
        }

        #[ink(message)]
        pub fn destroy_black_fund(&mut self, evil_user: AccountId) -> Result<()> {
            let caller = Self::env().caller();
            self.only_owner(caller)?;

            if self.is_blacklist(evil_user) {
                let evil_balance = self.balance_of_or_zero(&evil_user);
                self.balances.insert(evil_user, 0);
                *self.total_supply -= evil_balance;
            };
            Ok(())
        }

        #[ink(message)]
        pub fn is_blacklist(&self, id: AccountId) -> bool {
            match self.blacklist.get(&id) {
                Some(s) => *s,
                None => false,
            }
        }

        ///Increment total supply only by owner.
        #[ink(message)]
        pub fn inc_supply(&mut self, value: Balance) -> Result<()> {
            let caller = Self::env().caller();
            self.only_owner(caller)?;

            let owner_balance = self.balance_of_or_zero(&caller);
            *self.total_supply += value;
            self.balances.insert(caller, owner_balance + value);

            self.env().emit_event(IncrementSupply {
                from: *self.owner,
                value,
            });
            Ok(())
        }

        ///Decrement total supply only by owner.
        #[ink(message)]
        pub fn dec_supply(&mut self, value: Balance) -> Result<()> {
            let caller = Self::env().caller();
            self.only_owner(caller)?;

            let owner_balance = self.balance_of_or_zero(&caller);
            if owner_balance < value {
                return Err(Error::InsufficientBalance);
            }
            *self.total_supply -= value;
            self.balances.insert(caller, owner_balance - value);

            self.env().emit_event(DecrementSupply {
                from: *self.owner,
                value,
            });
            Ok(())
        }

        /// Returns the account balance for the specified `owner`.
        /// Returns `0` if the account is non-existent.
        #[ink(message)]
        pub fn balance_of(&self, owner: AccountId) -> Balance {
            self.balance_of_or_zero(&owner)
        }

        /// Allows `spender` to withdraw from the caller's account multiple times, up to
        /// the `value` amount.
        /// If this function is called again it overwrites the current allowance with `value`.
        /// An `Approval` event is emitted.
        #[ink(message)]
        pub fn approve(&mut self, spender: AccountId, value: Balance) -> bool {
            // Record the new allowance.
            let owner = self.env().caller();
            if self.is_blacklist(owner) {
                false
            } else {
                self.allowances.insert((owner, spender), value);

                // Notify offchain users of the approval and report success.
                self.env().emit_event(Approval {
                    owner,
                    spender,
                    value,
                });
                true
            }
        }

        /// Returns the amount which `spender` is still allowed to withdraw from `owner`.
        /// Returns `0` if no allowance has been set `0`.
        #[ink(message)]
        pub fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance {
            self.allowance_of_or_zero(&owner, &spender)
        }

        /// Transfers `value` tokens on the behalf of `from` to the account `to`.
        /// This can be used to allow a contract to transfer tokens on ones behalf and/or
        /// to charge fees in sub-currencies, for example.
        /// On success a `Transfer` event is emitted.
        /// # Errors
        /// Returns `InsufficientAllowance` error if there are not enough tokens allowed
        /// for the caller to withdraw from `from`.
        /// Returns `InsufficientBalance` error if there are not enough tokens on
        /// the the account balance of `from`.
        #[ink(message)]
        pub fn transfer_from(
            &mut self,
            from: AccountId,
            to: AccountId,
            value: Balance,
        ) -> Result<()> {
            // Ensure that a sufficient allowance exists.
            let caller = self.env().caller();
            let allowance = self.allowance_of_or_zero(&from, &caller);
            if allowance < value {
                return Err(Error::InsufficientAllowance);
            }

            self.transfer_from_to(from, to, value)?;
            self.allowances.insert((from, caller), allowance - value);
            Ok(())
        }

        /// Transfers `value` amount of tokens from the caller's account to account `to`.
        /// On success a `Transfer` event is emitted.
        /// Returns `InsufficientBalance` error if there are not enough tokens on
        /// the caller's account balance.
        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, value: Balance) -> Result<()> {
            self.transfer_from_to(self.env().caller(), to, value)
        }

        /// Transfers `value` amount of tokens from the caller's account to account `to`.
        /// On success a `Transfer` event is emitted.
        /// Returns `InsufficientBalance` error if there are not enough tokens on
        /// the caller's account balance.
        fn transfer_from_to(
            &mut self,
            from: AccountId,
            to: AccountId,
            value: Balance,
        ) -> Result<()> {
            if self.is_blacklist(from) || self.is_blacklist(to) {
                Err(Error::EvilAccount)
            } else {
                let from_balance = self.balance_of_or_zero(&from);
                if from_balance < value {
                    return Err(Error::InsufficientBalance);
                }

                // Update the sender's balance.
                self.balances.insert(from, from_balance - value);

                // Update the receiver's balance.
                let to_balance = self.balance_of_or_zero(&to);
                self.balances.insert(to, to_balance + value);

                self.env().emit_event(Transfer {
                    from: Some(from),
                    to: Some(to),
                    value,
                });
                Ok(())
            }
        }

        fn only_owner(&self, caller: AccountId) -> Result<()> {
            if *self.owner == caller {
                Ok(())
            } else {
                return Err(Error::InsufficientBalance);
            }
        }

        fn balance_of_or_zero(&self, owner: &AccountId) -> Balance {
            *self.balances.get(owner).unwrap_or(&0)
        }

        fn allowance_of_or_zero(&self, owner: &AccountId, spender: &AccountId) -> Balance {
            *self.allowances.get(&(*owner, *spender)).unwrap_or(&0)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        use ink_lang as ink;

        #[ink::test]
        fn new_works() {
            let contract = RielToken::new(777);
            assert_eq!(contract.total_supply(), 777);
        }

        #[ink::test]
        fn onlyowner_works() {
            let contract = RielToken::new(777);
            assert_eq!(contract.only_owner(AccountId::from([0x1; 32])), Ok(()));
        }

        #[ink::test]
        fn transfer_ownership_works() {
            let mut contract = RielToken::new(777);
            assert_eq!(contract.only_owner(AccountId::from([0x1; 32])), Ok(()));
            contract
                .transfer_ownership(AccountId::from([0x0; 32]))
                .unwrap();
            assert_eq!(contract.only_owner(AccountId::from([0x0; 32])), Ok(()));
        }

        #[ink::test]
        fn inc_subpply_works() {
            let mut contract = RielToken::new(777);
            contract.inc_supply(1000).unwrap();
            assert_eq!(contract.total_supply(), 1777);
            assert_eq!(contract.balance_of(AccountId::from([0x1; 32])), 1777);
        }

        #[ink::test]
        fn dec_subpply_works() {
            let mut contract = RielToken::new(777);
            contract.dec_supply(10).unwrap();
            assert_eq!(contract.total_supply(), 767);
            assert_eq!(contract.balance_of(AccountId::from([0x1; 32])), 767);
        }

        #[ink::test]
        fn balance_works() {
            let contract = RielToken::new(100);
            assert_eq!(contract.total_supply(), 100);
            assert_eq!(contract.balance_of(AccountId::from([0x1; 32])), 100);
            assert_eq!(contract.balance_of(AccountId::from([0x0; 32])), 0);
        }

        #[ink::test]
        fn transfer_works() {
            let mut contract = RielToken::new(100);
            assert_eq!(contract.balance_of(AccountId::from([0x1; 32])), 100);
            assert_eq!(contract.transfer(AccountId::from([0x0; 32]), 10), Ok(()));
            assert_eq!(contract.balance_of(AccountId::from([0x0; 32])), 10);
            assert_eq!(
                contract.transfer(AccountId::from([0x0; 32]), 100),
                Err(Error::InsufficientBalance)
            );
        }

        #[ink::test]
        fn transfer_from_works() {
            let mut contract = RielToken::new(100);
            assert_eq!(contract.balance_of(AccountId::from([0x1; 32])), 100);
            contract.approve(AccountId::from([0x1; 32]), 20);
            contract
                .transfer_from(AccountId::from([0x1; 32]), AccountId::from([0x0; 32]), 10)
                .unwrap();
            assert_eq!(contract.balance_of(AccountId::from([0x0; 32])), 10);
        }
    }
}
