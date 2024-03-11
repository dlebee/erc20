#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod erc20 {

    use ink::{storage::Mapping};

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct Erc20 {
        /// Stores a single `bool` value on the storage.
        total_supply: Balance,
        balances: Mapping<AccountId, Balance>,
        allowances: Mapping<(AccountId, AccountId), Balance>
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        InsufficientBalance,
        InsufficientAllowance
    }

    pub type Result<T> = core::result::Result<T, Error>;

    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        value: Balance
    }

    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        spender: AccountId,
        value: Balance
    }

    impl Erc20 {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new(initial_supply: Balance) -> Self {
            let caller = Self::env().caller();
            let mut mapping = Mapping::new();
            mapping.insert(&caller, &initial_supply);

            Self {
                total_supply: initial_supply,
                balances: mapping,
                allowances: Mapping::new()
            }
        }

        #[ink(message)]
        pub fn total_supply(&self) -> Balance {
            return self.total_supply
        }

        #[ink(message)]
        pub fn balance_of(&self, owner: AccountId) -> Balance {
            self.balance_of_impl(&owner)
        }

        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, value: Balance) -> Result<()> {
            let from = self.env().caller();
            self.transfer_from_to(&from, &to, value)
        }

        #[ink(message)]
        pub fn approve(&mut self, spender: AccountId, value: Balance) -> Result<()> {
            let owner = self.env().caller();
            self.allowances.insert((&owner, &spender), &value);
            self.env().emit_event(Approval{
                owner,
                spender,
                value
            });
            Ok(())
        }

        #[ink(message)]
        pub fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance {
            self.allowance_impl(&owner, &spender)
        }

        #[inline]
        pub fn allowance_impl(&self, owner: &AccountId, spender: &AccountId) -> Balance {
            self.allowances.get((owner, spender)).unwrap_or_default()
        }

        #[inline]
        pub fn balance_of_impl(&self, account: &AccountId) -> Balance {
            self.balances.get(account).unwrap_or_default()
        }

        #[ink(message)]
        pub fn transfer_from(&mut self, from: AccountId, to: AccountId, value: Balance) -> Result<()> {
            let caller = self.env().caller();
            let allowance = self.allowance_impl(&from, &caller);
            if allowance < value {
                return Err(Error::InsufficientAllowance)
            }

            self.transfer_from_to(&from, &to, value)?;
            self.allowances.insert((&from, &caller), &(allowance -value));
            Ok(())
        }

        pub fn transfer_from_to(&mut self, from: &AccountId, to: &AccountId, value: Balance) -> Result<()> {
            let from_balance = self.balance_of_impl(from);
            if from_balance < value {
                return Err(Error::InsufficientBalance)
            }

            self.balances.insert(from, &(from_balance-value));
            let to_balance = self.balance_of_impl(to);
            self.balances.insert(to, &(to_balance+value));

            self.env().emit_event(Transfer {
                from: Some(*from),
                to: Some(*to),
                value
            });

            Ok(())
        }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        use ink::primitives::AccountId;

        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// We test if the default constructor does its job.
        #[ink::test]
        fn new_works() {
            let erc20 = Erc20::new(1000);
            assert_eq!(erc20.total_supply(), 1000);
        }

        #[ink::test]
        fn balance_of_works() {
            let contract = Erc20::new(100);
            assert_eq!(contract.total_supply(), 100);
            assert_eq!(contract.balance_of(AccountId::from([0x1; 32])), 100);
            assert_eq!(contract.balance_of(AccountId::from([0x0; 32])), 0);
        }

        #[ink::test]
        fn transfer_works() {
            let mut contract = Erc20::new(100);

            let from = AccountId::from([0x1; 32]);
            let to = AccountId::from([0x0; 32]);

            assert_eq!(contract.balance_of(from), 100);
            assert_eq!(contract.balance_of(to), 0);
            assert_eq!(contract.transfer(to, 10), Ok(()));
            assert_eq!(contract.balance_of(from), 90);
            assert_eq!(contract.balance_of(to), 10);
        }

        #[ink::test]
        fn transfer_protected_from_moving_too_much_balance() {
            let mut contract = Erc20::new(100);

            let from = AccountId::from([0x1; 32]);
            let to = AccountId::from([0x0; 32]);

            assert_eq!(contract.balance_of(from), 100);
            assert_eq!(contract.balance_of(to), 0);
            assert_eq!(contract.transfer(to, 110), Err(Error::InsufficientBalance));
        }

        #[ink::test]
        fn transfer_from_works() {
            let mut contract = Erc20::new(100);

            let from = AccountId::from([0x1; 32]);
            let to = AccountId::from([0x0; 32]);

            assert_eq!(contract.balance_of(from), 100);
            assert_eq!(contract.approve(from, 20), Ok(()));
            assert_eq!(contract.transfer_from(from, to, 10), Ok(()));
            assert_eq!(contract.balance_of(to), 10);
        }

        #[ink::test]
        fn allowance_works() {
            let mut contract = Erc20::new(100);

            let x0 = AccountId::from([0x0; 32]);
            let x1 = AccountId::from([0x1; 32]);

            // create allowance and verify it worked.
            assert_eq!(contract.balance_of(x1), 100);
            assert_eq!(contract.approve(x1, 200), Ok(()));
            assert_eq!(contract.allowance(x1, x1), 200);

            // transfer from allowance.
            assert_eq!(contract.transfer_from(x1, x0, 50), Ok(()));
            assert_eq!(contract.balance_of(x0), 50);
            assert_eq!(contract.allowance(x1, x1), 150);

            // should not have enough allowance and balance should remain the same.
            assert_eq!(contract.transfer_from(x1, x0, 300), Err(Error::InsufficientAllowance));
            assert_eq!(contract.balance_of(x0), 50);
            assert_eq!(contract.allowance(x1, x1), 150);

            // should not have enough balance, but enough allowance.
            assert_eq!(contract.transfer_from(x1, x0, 100), Err(Error::InsufficientBalance));
            assert_eq!(contract.balance_of(x0), 50);
            assert_eq!(contract.allowance(x1, x1), 150);
        }
    }
}
