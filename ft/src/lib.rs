/*!
Fungible Token implementation with JSON serialization.
NOTES:
  - The maximum balance value is limited by U128 (2**128 - 1).
  - JSON calls should pass U128 as a base-10 string. E.g. "100".
  - The contract optimizes the inner trie structure by hashing account IDs. It will prevent some
    abuse of deep tries. Shouldn't be an issue, once NEAR clients implement full hashing of keys.
  - The contract tracks the change in storage before and after the call. If the storage increases,
    the contract requires the caller of the contract to attach enough deposit to the function call
    to cover the storage cost.
    This is done to prevent a denial of service attack on the contract by taking all available storage.
    If the storage decreases, the contract will issue a refund for the cost of the released storage.
    The unused tokens from the attached deposit are also refunded, so it's safe to
    attach more deposit than required.
  - To prevent the deployed contract from being modified or deleted, it should not have any access
    keys on its account.
*/
use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider, FT_METADATA_SPEC,
};
use near_contract_standards::fungible_token::FungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LazyOption;
use near_sdk::json_types::U128;
use near_sdk::{env, log, near_bindgen, AccountId, Balance, PanicOnDefault, PromiseOrValue};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    token: FungibleToken,
    metadata: LazyOption<FungibleTokenMetadata>,
}

const DATA_IMAGE_SVG_NEAR_ICON: &str = "data:image/svg+xml;base64,PD94bWwgdmVyc2lvbj0iMS4wIiBzdGFuZGFsb25lPSJubyI/Pgo8IURPQ1RZUEUgc3ZnIFBVQkxJQyAiLS8vVzNDLy9EVEQgU1ZHIDIwMDEwOTA0Ly9FTiIKICJodHRwOi8vd3d3LnczLm9yZy9UUi8yMDAxL1JFQy1TVkctMjAwMTA5MDQvRFREL3N2ZzEwLmR0ZCI+CjxzdmcgdmVyc2lvbj0iMS4wIiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciCiB3aWR0aD0iMTYxMy4wMDAwMDBwdCIgaGVpZ2h0PSIxNjEzLjAwMDAwMHB0IiB2aWV3Qm94PSIwIDAgMTYxMy4wMDAwMDAgMTYxMy4wMDAwMDAiCiBwcmVzZXJ2ZUFzcGVjdFJhdGlvPSJ4TWlkWU1pZCBtZWV0Ij4KCjxnIHRyYW5zZm9ybT0idHJhbnNsYXRlKDAuMDAwMDAwLDE2MTMuMDAwMDAwKSBzY2FsZSgwLjEwMDAwMCwtMC4xMDAwMDApIgpmaWxsPSIjMDAwMDAwIiBzdHJva2U9Im5vbmUiPgo8cGF0aCBkPSJNOTE4NSA3NTQwIGMtMTM5IC0yNSAtMjUyIC04OSAtMzE2IC0xNzggLTUzIC03NCAtNzMgLTE0NiAtNzMgLTI1MwoxIC03NCA1IC05NSAzMSAtMTUwIDU0IC0xMTcgMTU0IC0xOTAgMzU1IC0yNjMgMTQwIC01MSAyMDAgLTgwIDI0MSAtMTE2IDQ3Ci00MSA2MyAtOTEgNDggLTE0NyAtMjEgLTc2IC04OCAtMTA0IC0yNDYgLTEwNCAtMTE1IDEgLTIwMiAxNyAtMzAyIDU3IC0zNSAxNAotNjYgMjQgLTY3IDIyIC0xIC0xIC0yMCAtNTIgLTQxIC0xMTMgLTIxIC02MCAtNDAgLTExNiAtNDMgLTEyMyAtNiAtMTYgMTAwCi02NCAxOTAgLTg2IDIyMSAtNTUgNDkyIC00MCA2MzkgMzQgNjIgMzIgMTM1IDEwNSAxNjQgMTY0IDc3IDE2MSA0MCAzOTkgLTgxCjUwNSAtNTggNTEgLTE2NSAxMDYgLTMzNyAxNzEgLTE2OSA2NSAtMjE2IDEwMSAtMjI0IDE3MCAtNCAzNiAwIDUyIDE5IDgwIDMyCjQ4IDgxIDYzIDIwMyA2MyA4NSAwIDExMyAtNSAxODUgLTMwIDQ3IC0xNiA5MiAtMzMgMTAxIC0zNyAxMyAtNiAyNCAxNiA2MQoxMjAgbDQ2IDEyNiAtNTUgMjQgYy0xMDMgNDYgLTE5MSA2NSAtMzIzIDY5IC02OSAyIC0xNDcgMCAtMTc1IC01eiIvPgo8cGF0aCBkPSJNNjAzNCA3NTIwIGMtMzkgLTQgLTg5IC0xMSAtMTEyIC0xNSBsLTQyIC03IDAgLTY5OSAwIC02OTkgMjMgLTUKYzEyNiAtMjggNDA2IC00MiA1NDEgLTI1IDIyNSAyNyAzNjYgODcgNDg3IDIwOSAxMjMgMTI0IDE3NSAyNTQgMTg2IDQ2NyA3CjE0MiAtNyAyNTUgLTQ0IDM1OCAtNzUgMjA5IC0yMjUgMzM2IC00NjcgMzk3IC02NiAxNyAtMTIwIDIyIC0yOTEgMjQgLTExNSAyCi0yNDIgMCAtMjgxIC01eiBtNDk2IC0yODkgYzE2NyAtNTQgMjQ4IC0xNzcgMjU4IC0zOTIgNiAtMTM5IC0xNCAtMjM1IC02NwotMzE2IC04NyAtMTM1IC0yMDcgLTE4NiAtNDE5IC0xODEgbC0xMDcgMyAtMyA0NDkgYy0yIDM1MyAxIDQ1MiAxMCA0NTggMjQgMTcKMjU4IDEgMzI4IC0yMXoiLz4KPHBhdGggZD0iTTQ3NzQgNzQyMyBjLTEzMyAtMjg1IC0zMDIgLTcxNSAtNDcwIC0xMTkxIGwtNTMgLTE1MiAxNjYgMiAxNjYgMwo1MyAxNTAgNTIgMTUwIDI3OCAwIDI3OCAwIDUyIC0xNTAgNTIgLTE1MCAxNzEgLTMgYzk0IC0xIDE3MSAtMSAxNzEgMSAwIDEwCi0xNzQgNDg4IC0yNDAgNjYyIC03OCAyMDMgLTIwOSA1MTggLTI4NCA2NzggbC00NSA5NyAtMTUxIDAgLTE1MSAwIC00NSAtOTd6Cm0zMzcgLTYyMSBjMjkgLTc4IDQ5IC0xNDUgNDcgLTE0OCAtMyAtMiAtOTMgLTMgLTIwMCAtMiBsLTE5NSAzIDEwMSAyNjcgMTAwCjI2OCA0OCAtMTIzIGMyNiAtNjcgNzEgLTE4NiA5OSAtMjY1eiIvPgo8cGF0aCBkPSJNNzY5MyA3MzY4IGMtNzIgLTE1NyAtMjE4IC01MTQgLTI5OSAtNzMxIC03MCAtMTg3IC0xOTQgLTUzNyAtMTk0Ci01NDggMCAtNSA2OCAtOSAxNjMgLTkgbDE2MiAwIDU0IDE1NSA1MyAxNTUgMjc3IDAgMjc3IDAgNTMgLTE1NSA1NCAtMTU1IDE2OAowIGM5MyAwIDE2OSA0IDE2OSA4IDAgMTkgLTIwOSA1ODggLTI5NiA4MDYgLTk3IDI0MyAtMjQ4IDU5MCAtMjY2IDYxNCAtOCA4Ci01MiAxMiAtMTU4IDEyIGwtMTQ4IDAgLTY5IC0xNTJ6IG0yODEgLTM0OSBjMzEgLTgxIDc0IC0xOTcgOTYgLTI1OCBsMzkgLTExMQotMjAwIDAgYy0xNjIgMCAtMjAwIDMgLTE5NyAxMyAxMyA1NyAxOTQgNTIyIDE5OSA1MTMgNCAtNiAzMiAtNzcgNjMgLTE1N3oiLz4KPHBhdGggZD0iTTEwMDU3IDc1MTMgYy00IC0zIC03IC0zMjcgLTcgLTcyMCBsMCAtNzEzIDE2MCAwIDE2MCAwIDAgMzA1IDAgMzA1CjI3MCAwIDI3MCAwIDAgLTMwNSAwIC0zMDUgMTYwIDAgMTYwIDAgMCA3MjAgMCA3MjAgLTE2MCAwIC0xNjAgMCAwIC0yNzUgMAotMjc1IC0yNzAgMCAtMjcwIDAgMCAyNzUgMCAyNzUgLTE1MyAwIGMtODUgMCAtMTU3IC0zIC0xNjAgLTd6Ii8+CjxwYXRoIGQ9Ik0xMTU2MCA2ODAwIGwwIC03MjAgMTYwIDAgMTYwIDAgMCA3MjAgMCA3MjAgLTE2MCAwIC0xNjAgMCAwIC03MjB6Ii8+CjxwYXRoIGQ9Ik03ODQxIDU1MjggYy01IC0xMyAtNzUgLTE4OSAtMTU2IC0zOTMgLTgxIC0yMDMgLTE1MyAtMzg3IC0xNjEgLTQwNwpsLTEzIC0zOCA2MiAwIDYyIDAgMzEgODggMzEgODcgMTU4IDAgMTU4IDAgMjYgLTg1IDI3IC04NSA2MiAtMyBjMzQgLTIgNjIgMAo2MiAzIDAgNCAtMjczIDc1MiAtMzA2IDgzOCAtOSAyNCAtMzMgMjEgLTQzIC01eiBtNzggLTM4NSBjMjggLTg4IDUxIC0xNjggNTEKLTE3NyAwIC0xNCAtMTcgLTE2IC0xMjEgLTE2IC05MiAwIC0xMjAgMyAtMTE3IDEzIDE3IDY3IDEyMyAzNTYgMTI5IDM1MCA0IC01CjMwIC04MSA1OCAtMTcweiIvPgo8cGF0aCBkPSJNOTI2MCA1NTQxIGMtMTg0IC01NyAtMjkwIC0yNjQgLTI2MCAtNTEwIDE1IC0xMTggNTYgLTIwNyAxMjcgLTI3Mgo3MCAtNjMgMTM0IC04MyAyNDggLTc3IDg3IDQgMTcyIDM2IDIwOSA3NyAxNyAxOSAxNyAyMSAtMTAgNTggLTE2IDIxIC0yOSA0MAotMzEgNDIgLTEgMiAtMTggLTEwIC0zNyAtMjcgLTY1IC01NyAtMTYxIC02OCAtMjQzIC0yNyAtOTggNDkgLTE0NSAxNDQgLTE0NwoyOTUgLTEgMTY2IDUwIDI3NyAxNTAgMzI2IDQ4IDIzIDYzIDI2IDEyNyAyMiA0MSAtMyA4NyAtMTIgMTA0IC0yMCBsMzEgLTE2CjIxIDQ5IGMxMSAyNyAxNyA1MCAxMyA1MyAtNTAgMzAgLTIzNyA0NyAtMzAyIDI3eiIvPgo8cGF0aCBkPSJNNTkwMCA1MTE1IGwwIC00MjUgNTUgMCA1NSAwIDAgMjA1IDAgMjA1IDE1MCAwIDE1MCAwIDAgNTAgMCA1MAotMTUwIDAgLTE1MSAwIDMgMTE4IDMgMTE3IDIwMyAzIDIwMiAyIDAgNTAgMCA1MCAtMjYwIDAgLTI2MCAwIDAgLTQyNXoiLz4KPHBhdGggZD0iTTY1NDAgNTExNSBsMCAtNDI1IDU1IDAgNTUgMCAwIDQyNSAwIDQyNSAtNTUgMCAtNTUgMCAwIC00MjV6Ii8+CjxwYXRoIGQ9Ik02ODQwIDUxMTUgbDAgLTQyNSA1NSAwIDU1IDAgMCAzMDIgYzAgMjY0IDIgMzAwIDE0IDI4OCA4IC04IDEwNQotMTQ1IDIxNiAtMzA1IDEzMCAtMTg3IDIwOCAtMjkxIDIyMSAtMjkzIDE5IC0zIDE5IDcgMTkgNDI3IGwwIDQzMSAtNTUgMCAtNTUKMCAtMiAtMjkxIC0zIC0yOTEgLTIwOCAyOTEgYy0xNzYgMjQ2IC0yMTIgMjkxIC0yMzIgMjkxIGwtMjUgMCAwIC00MjV6Ii8+CjxwYXRoIGQ9Ik04MjkwIDUxMTYgbDAgLTQyNiA1MCAwIDUwIDAgMCAzMDAgYzAgMTY1IDMgMzAwIDggMjk5IDQgMCAxMDIgLTEzNwoyMTcgLTMwNCAxMzkgLTIwMiAyMTYgLTMwNSAyMjggLTMwNSAxNiAwIDE3IDI3IDE3IDQzMCBsMCA0MzAgLTU1IDAgLTU1IDAgLTIKLTI5MCAtMyAtMjkwIC0yMDYgMjg4IGMtMTQ4IDIwNyAtMjExIDI4OCAtMjI3IDI5MCBsLTIyIDMgMCAtNDI1eiIvPgo8cGF0aCBkPSJNOTc0MCA1MTE1IGwwIC00MjUgMjQ1IDAgMjQ1IDAgMCA1MCAwIDUwIC0xOTAgMCAtMTkwIDAgMCAxNTUgMCAxNTUKMTM1IDAgMTM1IDAgMCA1MCAwIDUwIC0xMzUgMCAtMTM1IDAgMCAxMjAgMCAxMjAgMTkwIDAgMTkwIDAgMCA1MCAwIDUwIC0yNDUKMCAtMjQ1IDAgMCAtNDI1eiIvPgo8L2c+Cjwvc3ZnPgo=";

#[near_bindgen]
impl Contract {
    /// Initializes the contract with the given total supply owned by the given `owner_id` with
    /// default metadata (for example purposes only).
    #[init]
    pub fn new_default_meta(owner_id: AccountId, total_supply: U128) -> Self {
        Self::new(
            owner_id,
            total_supply,
            FungibleTokenMetadata {
                spec: FT_METADATA_SPEC.to_string(),
                name: "Adashi Finance".to_string(),
                symbol: "ADH".to_string(),
                icon: Some(DATA_IMAGE_SVG_NEAR_ICON.to_string()),
                reference: None,
                reference_hash: None,
                decimals: 24,
            },
        )
    }

    /// Initializes the contract with the given total supply owned by the given `owner_id` with
    /// the given fungible token metadata.
    #[init]
    pub fn new(
        owner_id: AccountId,
        total_supply: U128,
        metadata: FungibleTokenMetadata,
    ) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        metadata.assert_valid();
        let mut this = Self {
            token: FungibleToken::new(b"a".to_vec()),
            metadata: LazyOption::new(b"m".to_vec(), Some(&metadata)),
        };
        this.token.internal_register_account(&owner_id);
        this.token.internal_deposit(&owner_id, total_supply.into());
        near_contract_standards::fungible_token::events::FtMint {
            owner_id: &owner_id,
            amount: &total_supply,
            memo: Some("Initial tokens supply is minted"),
        }
        .emit();
        this
    }

    fn on_account_closed(&mut self, account_id: AccountId, balance: Balance) {
        log!("Closed @{} with {}", account_id, balance);
    }

    fn on_tokens_burned(&mut self, account_id: AccountId, amount: Balance) {
        log!("Account @{} burned {}", account_id, amount);
    }
}

near_contract_standards::impl_fungible_token_core!(Contract, token, on_tokens_burned);
near_contract_standards::impl_fungible_token_storage!(Contract, token, on_account_closed);

#[near_bindgen]
impl FungibleTokenMetadataProvider for Contract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        self.metadata.get().unwrap()
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, Balance};

    use super::*;

    const TOTAL_SUPPLY: Balance = 1_000_000_000_000_000;

    fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    #[test]
    fn test_new() {
        let mut context = get_context(accounts(1));
        testing_env!(context.build());
        let contract = Contract::new_default_meta(accounts(1).into(), TOTAL_SUPPLY.into());
        testing_env!(context.is_view(true).build());
        assert_eq!(contract.ft_total_supply().0, TOTAL_SUPPLY);
        assert_eq!(contract.ft_balance_of(accounts(1)).0, TOTAL_SUPPLY);
    }

    #[test]
    #[should_panic(expected = "The contract is not initialized")]
    fn test_default() {
        let context = get_context(accounts(1));
        testing_env!(context.build());
        let _contract = Contract::default();
    }

    #[test]
    fn test_transfer() {
        let mut context = get_context(accounts(2));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(2).into(), TOTAL_SUPPLY.into());
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(contract.storage_balance_bounds().min.into())
            .predecessor_account_id(accounts(1))
            .build());
        // Paying for account registration, aka storage deposit
        contract.storage_deposit(None, None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(2))
            .build());
        let transfer_amount = TOTAL_SUPPLY / 3;
        contract.ft_transfer(accounts(1), transfer_amount.into(), None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        assert_eq!(contract.ft_balance_of(accounts(2)).0, (TOTAL_SUPPLY - transfer_amount));
        assert_eq!(contract.ft_balance_of(accounts(1)).0, transfer_amount);
    }
}
