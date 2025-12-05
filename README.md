# Pime

## What:
A Pinocchio time-based on-chain vault securing your assets all 
within your control, even if your private key is stolen.

## Why:
Secures assets owned by a wallet behind the only thing one can buy, time.
Various wallet owners, commonly on social media, are targeted by scams and threats.
Pime acts as a way to demotivate a threat from targeting you, 
similarly to how your home 
security system demotivates burglars due to the decreased risk/reward.

## How:
Your funds, regardless of native or tokenized, are stored as an asset owned by Pime.
Only a portion of your funds can be extracted within a certain time-frame, limiting the vault's outflow.
However, to accommodate larger transactions, the transaction has to be booked beforehand, with the ability to be canceled.

## Stack:
Pinocchio
Mollusk (Replace with LiteSVM?)
SurfPool


# Product-Market Fit
Crypto currencies, by its nature, enables anyone to control and transfer their assets without a third party being involved.
This means that any wallet can transfer any amount, of any on-chain assett, to anyone, at any time.
However, in recent years, this core concept has been used against people, who has seen their private keys compromised, and assets stolen.
When a wallet is compromised, by nature, all assets can be drained in one transaction, and be gone, with finality, in 400ms.
There are already solutions trying to reduce the posibility for a private key to get stole through physical ledgers, and password protections.
This is however not fool proof, especially against social hacking, or even kidnappings.
Pime indends to minimize the likelyness of you being a target by protecting you through the only thing one can't buy: Time.
Using Pime, all your assets are secured behind a time-based lock, where you can control how much and how fast your assets can be transfered.
Pime aims to not interfer with your day-to-day activities such as buying grocerys, and buying a movie ticked, but protect your wallet from being drained in milliseconds.
Moreover, occassionally, larger transaction may be needed, such as buying a car or a house, where larger transactions can be booked ahead of time, and unbooked at any point.
This is no different from how banks works today where a bank requires you to book a larger transaction with them and takes days until it's finilized, except that you are under control of your assets.
Understandably, if your wallet gets compromised it still can be drained, however at a limited rate.
This can enable you to take actions, both legally, but also controlling larger outflows so that you can recover your assets.
Pime will not stop attacks from happen, but demotivate bad actors from targeting you as Pime lowers the Risk/Reward, similarly to how your house alarm works.
Safety never prevents crime to happen, but give time for lawenforment to react, lowering the risk/reward. 

## UX Requirements
* Easy to use - A person why has little to no experience in Crypto should be able to understand and use it.
* Clarity - Users shall understand the core concepts in a few sentences without requiring technical background.

## Technical Requirements
* Decentralized - No third-party centralized dependencies, such as Oracles. (Maybe use oracles to set the vault limit by fiat instead of tokens as an option?)
* Lightweight - Pinocchio-based for low CU cost. Minimize user cost and Solana thoughput.
* Native and SPL token support - Any assets can be saved in the vault, Native just as SPL tokens (Including NFTs).

## Vocabulary
* Cooldown - Time until when a new withdraw can be performed
* Warmup - Time until a withdraw can be made from a new cooldown is set.
* Validity - Timeframe which something (like a transaction) is valid within.

# User Story
## General End State 
When a user is able to create new vaults. Each asset type (native or SPL) can be stored in a vault, where the user is able to create multiple vaults per asset.
Each vault has its own outflow restriction, and is determined by the user upon creation.
Moreover, the user is able to book larger transactions to a target wallet, where the supplied warmup (min 72h(?.. idk why, better solutions?)) must pass before the transaction can be executed.
When a transaction is booked, the user's assets from the specified vault are moved to a deposit wallet, and when the transaction is executed, the funds are sent to the target wallet.
If more than its validity has passed from when the warmup timer passed, the transaction needs to be closed, and a new one to be booked.
The user can close both vaults and transactions, where the vault must be empty or below the supplied threashold so that the remaining funds can be sent back to the original wallet.

## Actors
### A regular user who wants to store their assets
#### Vault
* As a new user, when I make a new vault, the vault is created along with the vault's restrictions.
* As a new user, when I create another wallet using a different vault index but same asset type, a new vault is created along with the vault's restrictions.
* As a vault owner, when I deposit assets to the vault, the assets are moved to the vault.
* As a vault owner, when I withdraw assets from a vault, the assets are moved back to the owners "original" pubkey for the asset.
* As a vault owner, if I withdraw more times than the limit of the vault allows, no assets will be withdrawn.
* As a vault owner, if I withdraw more assets than the limit of the vault allows, no assets will be withdrawn.
#### Transfer
* As a vault owner, if I book a transaction, a transaction is booked on-chain, and the assets are moved to a deposit.
* As a vault owner, if I try to transfer a booked transaction before the warmup has passed, nothing will happen.
* As a vault owner, if I try to transfer a booked transaction after the transaction's validity timeframe, nothing will happen.

### A regular on-chain user who want to send assets to a vault.
#### Vault
* As an on-chain user, when I transfer assets to an existing vault, the assets will be transferred.
* As an on-chain user, when I transfer assets to an non-existing vault, the vault will be initialized and assets transferred WITHOUT initializing the vault data.
* As an on-chain user, when I 
#### Transfer

### An attacker without a vault owners private key
#### Vault
* As an attacker, if I try to create a vault data, nothing will happen.
* As an attacker, if I try to withdraw assets to a different wallet than the original, nothing will happen.
* As an attacker, if I try to close a vault, nothing will happen.
* As an attacker, if I try to close a vault data, nothing will happen.
#### Transfer
* As an attacker, if I try to withdraw assets from a deposited transaction, nothing will happen.
* As an attacker, if I try to unbook a trans

### An attacker with a vault owners private key
*

# Phase 1
## Deadline 8 Dec -25
* Complete user stories.

## Deadline 15 Dec -25
* Completed architecture diagram.

## Deadline 1.1.26
* Ability to create vault.
* Ability to remove vault.
* Ability to change vault settings.
* Ability to deposit to vault.
* Ability to withdraw from vault, with restrictions.

Look into:
* Magic block - Cheap transactions.
* LiteSVM - Simpler tests still using rust.
* Crowdfunding using booked transactions? Stretching its usability maybe..?

# Comments during iterations
* *IMPORTANT COMMENT* if the private key is stolen, the attacker can still access your assets.
* * Answer: This is to minimize the likelyhood of being targeted, sacrificing 
some convinience. (Is there a way to do this on-chain?)
* *IMPORTANT QUESTION* Is there a solution to verify that the person who access the wallet is 
the real owner of the wallet?
* * Answer: Since the plan is to at this stage completely do this on-chain and not a 
centralized off-chain solution, no.
* *Comment/suggestion* Use LiteSVM for testing, easier than Mollusk.
* *Question* Are vaults and transfers seperated.
* *Question* Who are the users: regular people, not businesses.
* *Question* How the rules are enforced.
