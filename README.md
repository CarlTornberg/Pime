# Pime

## Letter of Intent
### What:
A Pinocchio time-based on-chain vault securing your assets within your control, even if your private key is stolen.

### Why:
Secures assets owned by a wallet behind the only thing one can buy, time.
Various wallet owners, commonly on social media, are targeted by scams and threats.
Pime acts as a way to demotivate a threat from targeting you, similarly to how your home security system demotivates burglars due to the decreased risk/reward.

### How:
Your funds, regardless of native or tokenized, are stored as an asset owned by Pime.
Only a portion of your funds can be extracted within a certain time-frame, limiting the vault's outflow.
However, to accommodate larger transactions, the transaction has to be booked beforehand, with the ability to be canceled.

### Stack:
* Pinocchio
* Mollusk (Replace with LiteSVM?)
* SurfPool

## Overview
### Product-Market Fit
Crypto currencies, by default, enables anyone to own their assets with full control over them.
This includes the ability to transfer your assets, at any point, to anyone, without involving any third parties.
However, in recent years, this core concept has been used against people, who has seen their private keys compromised, and assets stolen.
When a wallet is compromised, by nature, all assets can be drained in one transaction, and be gone, with finality, in 400ms.
There are already solutions trying to reduce the possibility for a private key to get stole through physical ledgers, and password protections.
This is however not fool proof, especially against social hacking, or even kidnappings.
Pime intends to minimize the likeliness of you being a target by protecting you through the only thing one can't buy: Time.
Using Pime, all your assets are secured behind a time-based lock, where you can control how much and how fast your assets can be transferred.
Pime aims to not interfere with your day-to-day activities such as buying groceries, and buying a movie ticked, but protect your wallet from being drained in milliseconds.
Moreover, occasionally, larger transaction may be needed, such as buying a car or a house, where larger transactions can be booked ahead of time, and unbooked at any point.
This is no different from how banks works today where a bank requires you to book a larger transaction with them and takes days until it's finalized, except that you are under control of your assets.
Understandably, if your wallet gets compromised it still can be drained, however at a limited rate.
This can enable you to take actions, both legally, but also controlling larger outflows so that you can recover your assets.
Pime will not stop attacks from happen, but demotivate bad actors from targeting you as Pime lowers the Risk/Reward, similarly to how your house alarm works.
Safety never prevents crime to happen, but give time for law enforcement to react, lowering the risk/reward. 

### UX Requirements
* Easy to use - A person why has little to no experience in Crypto should be able to understand and use it.
* Clarity - Users shall understand the core concepts in a few sentences without requiring technical background.

### Technical Requirements
* Decentralized - No third-party centralized dependencies, such as Oracles. (Maybe use oracles to set the vault limit by fiat instead of tokens as an option?)
* Lightweight - Pinocchio-based for low CU cost. Minimize user cost and Solana throughput.
* Native and SPL token support - Any assets can be saved in the vault, Native just as SPL tokens (Including NFT's (Stage 2)).

### Vocabulary
 Vault - An on-chain vault owned by Pime, controlled by its authority.
* Timeframe - Timeframe of a vault's restrictions.
* Transaction - Transactions between a vault and an address, with limited time frame outflow.
* Transfer - Larger transaction from a vault to an address, requiring additional timeframe safeguards.
* Cool-down - Time until when a new withdraw can be performed.
* Warm-up - Time until a withdraw can be made from a new cool-down is set.
* Validity - Timeframe which something (like a transaction) is valid within.

## User Story
### General End State 
A user is able to create new vaults. 
An asset can be stored in a vault, where the user is able to create multiple vaults per asset.
Each vault has its own outflow restriction, and is determined by the user upon creation.
Moreover, the user is able to book larger transactions to a target wallet, where a warm-up period must pass before execution.
When a transaction is booked, the user's assets from the specified vault are moved to a deposit vault, and when the transaction is executed, the funds are sent to the target wallet.

### Actors
#### A regular user who wants to store their assets (Vault owner)
##### Vault
* As a new user, when I make a new vault, the provided index and mint creates a unique vault with vault specific outflow restrictions.
* As a vault owner, when I deposit assets to the vault, the assets are moved to the vault.
* As a vault owner, when I withdraw assets from a vault, the assets are moved back to the vault's owner.
##### Transfer
* As a vault owner, if I book a transfer, the transfer is booked on-chain, and the assets are moved to a deposit with a set warm-up period.
* As a vault owner, if I execute a transfer after the warm-up period has passed, the assets are transferred.
* As a vault owner, if I unbook a booked transfer, the assets are moved back to the original vault.

#### A regular on-chain user (Not a vault owner)
##### Vault
* As an on-chain user, when I transfer assets to an existing vault, the assets will be transferred to the vault.


## Timeline
### Deadline 8 Dec -25
* Complete user stories. (Done)

### Deadline 15 Dec -25
* Completed architecture diagram.

### Deadline 1.1.26
* Ability to create vault.
* Ability to remove vault.
* Ability to change vault settings.
* Ability to deposit to vault.
* Ability to withdraw from vault, with restrictions.

## Look into:
* Magic block - Cheap transactions.
* LiteSVM - Simpler tests still using rust.
* Crowdfunding using booked transactions? Stretching its usability maybe..?

## Comments during iterations
* *IMPORTANT COMMENT* if the private key is stolen, the attacker can still access your assets.
* * Answer: Pime aims to minimize the likelihood of being targeted by reducing an attackers risk/reward ration, by trading convenience by restricting a vaults outflow. (Is there a way to do this on-chain?)
* *IMPORTANT QUESTION* Is there a solution to verify that the person who access the wallet is the real owner of the wallet?
* * Answer: Yes, though manual centralized checks or other off-chain solutions. However, Pime is completely decentralized on-chain solution.
* *Comment/suggestion* Use LiteSVM for testing, easier than Mollusk.
* *Question* Are vaults and transfers separated. Answer: Yes.
* *Question* Who are the users? Answer: regular people, not businesses.
* *Question* How the rules are enforced. Answer: Though the on-chain program. Vault's keep track of their usage and rules, settings their own restrictions.

## Self notes:
* NFT's may requires a special case and handled differently than other SPL tokens due to its implementation and may not be suitable for how vaults works. (Or just let the vault owner set the outflow to 0, forcing the NFT to be transferred using a booked transfer)
