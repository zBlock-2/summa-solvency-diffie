# Analayize Summa's Merkle Sum Tree

## Circuit

Summa have halo2 circuit to verify Merkle Sum Tree's inclusion proof. To understand circuit, we will go through how does composed chips enable constraint for proper value of cells in circuit.

`MstInclusionCircuit` is composed with 3 chips : 1) MerkleSumTreeChip, 2) PoseidonChip, 3) RangeCheckChip

### MerkleSumTreeChip

**bool constraint**: `vec![s * swap_bit.clone() * (Expression::Constant(Fp::from(1)) - swap_bit)]`,
swap_bit \* ( 1- swap_bit ) = 0 as a constraint (= swap_bit should be 0 or 1)

| col_a | col_b | col_c    | bool_and_swap_selector | sum_selector |
| ----- | ----- | -------- | ---------------------- | ------------ |
| -     | -     | swap_bit | s                      | -            |

**swap constraint**: `vec![swap_constraint_1, swap_constraint_2]`

- first constraint: `(element_r_cur.clone() - element_l_cur.clone()) * swap_bit.clone() + element_l_cur.clone() = element_l_next`
  - if swap_bit =0, l_cur = l_next
  - if swap_bit =1, r_cur = l_next
- second constraint: `element_r_next = (element_l_cur - element_r_cur)*swap_bit + element_r_cur`
  - if swap_bit =0, r_cur = r_next
  - if swap_bit =1, l_cur = r_next

| col_a          | col_b         | col_c    | bool_and_swap_selector | sum_selector |
| -------------- | ------------- | -------- | ---------------------- | ------------ |
| element_l_cur  | element_r_cur | swap_bit | s                      | -            |
| element_l_next | right_balance | \_       | -                      | -            |

**sum constraint**: `s * (left_balance + right_balance - computed_sum)` with N elements (N is currencies)

- left_balance + right_balance = computed_sum

| col_a        | col_b          | col_c        | bool_and_swap_selector | sum_selector |
| ------------ | -------------- | ------------ | ---------------------- | ------------ |
| left_balance | element_r_next | computed_sum | -                      | s            |
| left_balance | element_r_next | computed_sum | -                      | s            |
| left_balance | element_r_next | computed_sum | -                      | s            |

< Assign Region >

**sum nodes balances per currency**

- current_balance is from prev level

| a                 | b                 | c                 | bool_and_swap_selector | sum_selector |
| ----------------- | ----------------- | ----------------- | ---------------------- | ------------ |
| `current_balance` | `element_balance` | `sum of balances` | -                      | s            |

### PoseidonChip :

This chip used `halo2_gadgets::poseidon::Pow5Config` to construct table.

- `poseidon_entry_chip` (Poseidon chip used to hash leaf node)

  - Width: 2, Rate: 1, L: N_CURRENCIES + 1
  - N + 1, is because leaf node hash contain username

- `poseidon_middle_chip` (Poseidon chip used to hash middle node)
  - Width: 2, Rate: 1, L: N_CURRENCIES + 2
  - N + 2, is because middle node hash contain LeftChild.hash and RightChild.hash

The definition below is from comment in Summa code.

- `S`: The specification for the Poseidon hash function,
- `WIDTH`: The width of the Poseidon permutation,
- `RATE`: The rate of the Poseidon permutation, typically WIDTH - 1.
- `L`: The length of the input array to the Poseidon hash function.
- `N_CURRENCIES`: The number of currencies for which the solvency is verified.

### RangeCheckChip :

lookup_enable_selector \* diff, u8_range

| z (advice) | lookup_enable_selector(selector) | lookup_u8_table(fixed) |
| ---------- | -------------------------------- | ---------------------- |
| z_cur      | s                                | u8_range               |
| z_next     | -                                | -                      |

### synthesize

1. **Leaf Node Hash**: leaf node of each user should be calculated into `H(username, balance[0], balance[1], ..., balance[N_ASSETS - 1])`format as `H` using poseidon hash.

```rust
// compute the entry hash
let mut current_hash = poseidon_entry_chip.hash(
    layouter.namespace(|| "perform poseidon entry hash"),
    entry_hasher_input,
)?;

// expose the first current hash, namely the leaf hash, as public input
self.expose_public(
    layouter.namespace(|| "public leaf hash"),
    &current_hash,
    0,
    config.instance,
)?;
```

In this code base, what poseidon entry chip compute `Poseiodn(username, balance[0], balance[1], ..., balance[N_ASSETS - 1])` and expose the result as public input ( Instance column ).

| Instance(output)          | advice_a       | advice_b                                       |
| ------------------------- | -------------- | ---------------------------------------------- |
| -                         | entry username | entry_balance (currency 0)                     |
| -                         | \_             | entry_balance (currency 1)                     |
| -                         | \_             | ... base on currency types, it will keep stack |
| -                         | \_             | entry_balance (currency N - 1)                 |
| poseidon hash (leaf hash) | -              | -                                              |

2.  **sibling leaf hash**: While looping merkle sum tree each level.

- Level 0
  - 1. assign sibling leaf node ( hash & balance)
  - 2. compute sibling leaf hash in poseidon entry chip
  - 3. range check for leaf node balance and sibling node balance
  - 4. (common) assign swap bit and swap hash
- Level N ( N > 0 )
  - 1. assign sibling middle node ( hash & balance )
  - 2.  compute sibling middle hash in poseidon middle chip
  - 3. range check for sibling node balance
  - 4. (common) assign swap bit and swap hash

## Summa's Merkle Sum Tree Implementation

Summa's implementation of the Merkle Sum Tree is designed for higher efficiency and addresses the vulnerabilities found in the Original Merkle Sum Tree, as described in a particular paper. It includes zkProofs for verifying tree integrity without revealing user details and an efficient approach for hashing in the middle nodes.

### Middle Node Hash Comparison

| **Attribute** | **Broken Merkle Tree**                                                                                                                                                                              | **Summa's Approach**                                                                                                                                                                                      |
| ------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Formula**   | `H(LeftChild.hash, LeftChild.balance[0], LeftChild.balance[1], ..., LeftChild.balance[N_ASSETS], RightChild.hash, RightChild.balance[0], RightChild.balance[1], ..., RightChild.balance[N_ASSETS])` | `H(LeftChild.balance[0] + RightChild.balance[0], LeftChild.balance[1] + RightChild.balance[1], ..., LeftChild.balance[N_ASSETS - 1] + RightChild.balance[N_ASSETS - 1], LeftChild.hash, RightChild.hash)` |
| **Size**      | `2 * (1 + N_ASSETS)`                                                                                                                                                                                | `N_ASSETS + 2`                                                                                                                                                                                            |

Broken Merkel Sum Tree is a solution about Original Merkle Sum Tree’s vulnerability from this [paper](https://eprint.iacr.org/2022/043.pdf). As means, it have zkProof for verify tree’s integrity without revealing detail info about user(solution 2) and implemented middle node’s hash in hash(vl || vr || || h(l) || h(r)) this format (solution 1). However in terms of efficiency, Summa takes a unique approach to hash.

Compared to the original middle node hash described on paper as Broken Merkle Tree, for shorter hashing and cost for building the tree, building the zk proof of inclusion, Summa’s middle node hash formula is designed differently in this [PR](https://github.com/summa-dev/summa-solvency/issues/166).

### Summa's MST Implementation Details

#### Leaf Node

| **Attribute** | **Formula**                                                       |
| ------------- | ----------------------------------------------------------------- |
| **Hash**      | `H(username, balance[0], balance[1], ..., balance[N_ASSETS - 1])` |
| **Balance**   | `balance[0], balance[1], ..., balance[N_ASSETS - 1]`              |

#### Middle Node

| **Attribute** | **Formula**                                                                                                                                                                                               |
| ------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Hash**      | `H(LeftChild.balance[0] + RightChild.balance[0], LeftChild.balance[1] + RightChild.balance[1], ..., LeftChild.balance[N_ASSETS - 1] + RightChild.balance[N_ASSETS - 1], LeftChild.hash, RightChild.hash)` |
| **Balance**   | `LeftChild.balance[0] + RightChild.balance[0], LeftChild.balance[1] + RightChild.balance[1], ..., LeftChild.balance[N_ASSETS - 1] + RightChild.balance[N_ASSETS - 1]`                                     |
