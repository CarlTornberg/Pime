### Actors
#### A regular user who wants to store their assets (Vault owner)
##### Vault
* As a new user, when I make a new vault, the vault is created along with the vault's restrictions.
* As a new user, when I create another wallet using a different vault index but same asset type, a new vault is created along with the vault's restrictions.
* As a vault owner, when I deposit assets to the vault, the assets are moved to the vault.
* As a vault owner, when I withdraw assets from a vault, the assets are moved back to the owners "original" pubkey for the asset.
* As a vault owner, if I withdraw more times than the limit of the vault allows, no assets will be withdrawn.
* As a vault owner, if I withdraw more assets than the limit of the vault allows, no assets will be withdrawn.
##### Transfer
* As a vault owner, if I book a transfer, the transfer is booked on-chain, and the assets are moved to a deposit.
* As a vault owner, if I try make a new transfer when a transfer already exists, nothing will happen.
* As a vault owner, if I try to transfer a booked transaction before the warmup has passed, nothing will happen.
* As a vault owner, if I try to transfer a booked transaction after the transaction's validity timeframe, nothing will happen.
* As a vault owner, if I unbook an transfer under warmup, the assets are transferred back to the vault owner and the transfer closed.
* As a vault owner, if I unbook an active transfer, the assets are transferred back to the vault owner and the transfer closed.
* As a vault owner, if I unbook an expired transfer, the assets are transferred back to the vault owner and the transfer closed.

#### A regular on-chain user (Not a vault owner)
##### Vault
* As an on-chain user, when I transfer assets to an existing vault, the assets will be transferred.
* As an on-chain user, when I transfer assets to an non-existing vault, the vault will be initialized and assets transferred WITHOUT initializing the vault data.
##### Transfer (Only system program
* As an on-chain user, when I transfer assets to an non booked transfer, the assets are transferred (system program interaction only)
* As an on-chain user, when I transfer assets to an booked transfer, the assets are transferred (system program interaction)
* As an on-chain user, when I transfer assets to an transfer under warmup, the assets are transferred (system program interaction)
* As an on-chain user, when I transfer assets to an expired transfer, the assets are transferred (system program interaction)

#### An attacker without a vault owners private key
##### Vault
* As an attacker, if I try to create a vault data, nothing will happen.
* As an attacker, if I try to withdraw assets to a different wallet than the original, nothing will happen.
* As an attacker, if I try to close a vault, nothing will happen.
* As an attacker, if I try to close a vault data, nothing will happen.
##### Transfer
* As an attacker, if I try to withdraw assets from a deposited transaction, nothing will happen.
* As an attacker, if I try to unbook a transfer, nothing will happen.

#### An attacker with a vault owners private key
##### Vault
* As an attacker, if I try to drain a vault to different address than what created it, nothing will happen.
* As an attacker, if I try to drain a vault back to the authority wallet, the vault's time-frame threshold can not be exceeded.
##### Transfer
* As an attacker, if I try to book a transfer below the vaults min timeframe, nothing will happen.
* As an attacker, if I try to book a transfer within the vault min timeframe, the transaction can be cancelled.
