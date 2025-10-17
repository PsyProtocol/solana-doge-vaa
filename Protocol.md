
# VAA P2SH Protocol

## 1. Introduction

The VAA Pay-to-Script-Hash (VAA-P2SH) protocol represents a significant leap forward in bridging UTXO-based cryptocurrencies like Dogecoin and Bitcoin to smart contract ecosystems. Developed through a collaboration between Psy Protocol and Wormhole, this protocol establishes a trust-minimized, scalable, and highly secure method for smart contracts on Solana to custody and manage funds on the Dogecoin network.

The core innovation is the embedding of Wormhole VAA (Verified Action Approval) metadata directly into a Dogecoin P2SH (Pay-to-Script-Hash) locking script. This creates a cryptographic link between a specific Dogecoin address and its originating smart contract, allowing Wormhole Guardians to enforce control flow with unparalleled security.

## 2. Goals and Advantages

The VAA-P2SH protocol was designed to overcome the traditional challenges of bridging UTXO assets:

*   **Eliminate Guardian Key Management:** In many bridge designs, validators must manage a vast number of private keys, one for each user-deposited wallet. VAA-P2SH allows the entire Wormhole Guardian network to secure an unlimited number of Dogecoin wallets using a **single, shared Threshold Signature Scheme (TSS) public key**. This drastically reduces complexity and attack surface.
*   **Cryptographic Segregation of Funds:** The protocol ensures that funds deposited on behalf of one smart contract can *only* be spent by an instruction (a VAA) originating from that same contract. It is cryptographically impossible for Guardians to misappropriate funds by using a VAA from a different source.
*   **Infinite Scalability:** A single smart contract on Solana can control a virtually unlimited number of unique Dogecoin wallets by simply specifying a unique `sub_address_seed`. This is ideal for applications like exchanges or protocols that require segregated user deposit addresses.
*   **Trust-Minimized Security:** The security of the bridge relies on the proven economic and cryptographic security of the Wormhole Guardian network and their TSS implementation, not on novel or unaudited multi-party computation schemes. The logic is enforced on-chain by the Dogecoin script itself.

## 3. How It Works: The VAA-P2SH Script

The magic of the protocol lies in the `redeemScript` of a P2SH address. When someone sends funds to a VAA-P2SH address, they are locking those funds to a script that only the Wormhole Guardians can solve, and only under specific conditions.

### The Redeem Script

Here is the structure of the VAA-P2SH `redeemScript`, with human-readable explanations:

```
// === Start of Wormhole VAA Header ===

// 1. Push the VAA emitter_chain ID (e.g., 1 for Solana)
<EMITTER_CHAIN>

// 2. Push the 32-byte VAA emitter_contract_address
OP_PUSHBYTES_32 <EMITTER_CONTRACT>

// 3. Drop the chain and contract from the stack. They have served their purpose:
//    to be included in the script's hash, thus creating a unique address.
OP_2DROP

// 4. Push the 32-byte sub_address_seed. This allows one contract
//    to control multiple unique addresses.
OP_PUSHBYTES_32 <SUB_ADDRESS_SEED>

// 5. Drop the seed. It has also served its purpose.
OP_DROP

// === End of Wormhole VAA Header ===

// === Standard P2PKH (Pay-to-Public-Key-Hash) Script ===

// 6. Duplicate the Guardian TSS Public Key that will be provided in the input
OP_DUP

// 7. Hash the public key (RIPEMD160(SHA256(pubkey)))
OP_HASH160

// 8. Push the known hash of the Guardian TSS Public Key
OP_PUSHBYTES_20 <GUARDIAN_TSS_PUBLIC_KEY_HASH>

// 9. Verify that the provided public key hashes to the correct, known value
OP_EQUALVERIFY

// 10. Verify that the transaction signature is valid for the provided public key
OP_CHECKSIG
```

### Script Execution Flow

1.  **Locking Funds (Deposit):**
    *   A smart contract on Solana (e.g., `14gr...hhZ5`) generates a unique Dogecoin address for a user. It does this by constructing the `redeemScript` above using its own chain ID (`1`), its own address (`14gr...hhZ5`), and a unique seed for the user (`0x00...00`).
    *   It then hashes this `redeemScript` (`RIPEMD160(SHA256(redeemScript))`) to get a script hash.
    *   This script hash is converted into a standard P2SH address (e.g., `9xWK...HgT`).
    *   The user deposits Dogecoin to this address.

2.  **Unlocking Funds (Withdrawal):**
    *   To spend these funds, the Solana contract emits a Wormhole message (a VAA) specifying the withdrawal details (destination, amount).
    *   The Wormhole Guardians observe this VAA. They parse the `emitter_chain`, `emitter_contract`, and `sub_address_seed` from the VAA payload.
    *   The Guardians use this data to reconstruct the exact same `redeemScript` that locks the funds. This is how they know which UTXOs to spend.
    *   They construct a new Dogecoin transaction according to the VAA's instructions.
    *   For each input they want to spend, they generate a **Signature Hash (Sighash)**. **This is the critical step.** The sighash preimage includes the `redeemScript`.
    *   The Guardians use their TSS protocol to collectively sign this sighash.
    *   They create the unlocking script (`scriptSig`) for the transaction input, which consists of:
        *   The collective TSS signature.
        *   The shared Guardian TSS public key.
        *   The full `redeemScript` itself.
    *   When the Dogecoin network processes the transaction, it first verifies that the hash of the provided `redeemScript` matches the script hash in the UTXO being spent. Then, it executes the `redeemScript`:
        *   The VAA header opcodes (`<DATA>`, `OP_2DROP`, etc.) execute, leaving the stack clean. Their only purpose was to be part of the script's hash.
        *   The rest of the script executes exactly like a standard P2PKH payment, verifying the Guardians' public key and signature.

## 4. Cryptographic Guarantees and Usecases

The design of the VAA-P2SH script provides powerful, cryptographically-enforced guarantees.

#### Guarantee 1: Source Specificity

The `emitter_chain` and `emitter_contract_address` are baked into the address itself.

*   **Usecase:** A Psy Protocol smart contract on Solana can create a Dogecoin address that *only it* can control. If another, malicious contract on Solana tries to emit a VAA to spend these funds, the Guardians would reconstruct a *different* `redeemScript` (with the malicious contract's address), which would lead to a different P2SH address. They would not find any UTXOs at that new address, and the withdrawal would fail. The Guardians are physically unable to spend funds from Contract A's address using instructions from Contract B.

#### Guarantee 2: Address Space Virtualization

The `sub_address_seed` allows a single emitter contract to generate a near-infinite number of unique deposit addresses.

*   **Usecase:** A decentralized exchange can provide a unique, non-custodial Dogecoin deposit address for every single user. When a user deposits, the exchange's Solana contract can instantly credit their account. All these funds are secured by the same Guardian TSS key, but are logically segregated on the Dogecoin chain. This avoids the accounting complexities and security risks of a single large hot wallet.

#### Guarantee 3: Message Integrity

The signature produced by the Guardians via `OP_CHECKSIG` covers the entire transaction structure (inputs, outputs, amounts), which is derived from the VAA payload.

*   **Usecase:** When a user requests to withdraw 100 DOGE, the VAA will specify an output of 100 DOGE. The Guardians construct the transaction with this output. The resulting sighash depends on this output. The final signature cryptographically commits the Guardians to this specific transaction. They cannot change the amount to 99 DOGE and pocket the difference, as this would invalidate the signature.

## 5. Conclusion

The VAA-P2SH protocol is a powerful primitive for cross-chain communication. By moving VAA validation into the UTXO script itself, it provides a robust framework for building highly secure and scalable bridges. This design minimizes trust assumptions, reduces operational overhead for validators, and unlocks new possibilities for integrating Dogecoin and other UTXO chains into the decentralized finance ecosystem.
