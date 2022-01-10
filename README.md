# Avalanche Attack on Proof-of-Stake GHOST

Authors: Joachim Neu, Ertem Nusret Tas, David Tse


## High Level

The Avalanche Attack on PoS (Proof-of-Stake) GHOST (Greedy Heaviest Observed Sub-Tree) combines *selfish mining* with *equivocations*. The adversary uses withheld blocks to displace an honest chain once it catches up in sub-tree weight with the number of withheld adversarial blocks. The withheld blocks are released in a flat but wide sub-tree, exploiting the fact that under the GHOST rule such a sub-tree can displace a long chain. Only two withheld blocks enter the canonical chain permanently, while the other withheld blocks can subsequently be reused (through equivocations) to build further sub-trees to displace even more honest blocks. The attack exploits a specific weakness of the GHOST rule in combination with equivocations from PoS. Formal security proof of PoS GHOST seems doomed.

For details, see the proof-of-concept implementation code.


## Proof-of-Concept

A proof-of-concept implementation is provided in [`ghost-avalanche-attack`](/ghost-avalanche-attack/). It provides settings to simulate the attack for vanilla PoS GHOST and the Committee-GHOST variant proposed for PoS Ethereum.


## Results

We plot a snapshot of the block tree (adversarial blocks: red, honest blocks: green) resulting after 100 time slots. The attack is still ongoing thereafter, and as long as the attack is sustained, no honest blocks remain in the canonical chain permanently.

### PoS GHOST

* Adversarial stake: 30%
* Initially withheld adversarial blocks: 4

![](/ghost-avalanche-attack/attack-pos-ghost.png)

### Committee-GHOST

* Adversarial stake: 20%
* Initially withheld adversarial blocks: 12

![](/ghost-avalanche-attack/attack-committee-ghost.png)


## Attack Details

Selfish mining and equivocations can be used to attack PoS GHOST (using an 'avalanche of equivocating sub-trees rolling over honest chains' -- hence the name of the attack). The following description is for vanilla PoS GHOST, but can be straightforwardly translated for Committee-GHOST. Variants of this attack work for Committee-GHOST with Proposal Weights as well.

Suppose an adversary gets $k$ block production opportunities in a row, for modest $k$. The adversary withholds these $k$ blocks, as in *selfish mining*. On average, more honest blocks are produced than adversary blocks, so the developing honest chain eventually 'catches up' with the $k$ withheld adversarial blocks.

In that moment, the adversary releases the $k$ withheld blocks. However, not on a competing adversarial chain (as in selfish mining for a Longest Chain protocol), but on a competing adversarial sub-tree of height 2, where all but the first withheld block are siblings off the first withheld block. Due to the GHOST weight counting, this adversarial sub-tree is now of equal weight as the honest chain -- so the honest chain is abandoned.

At the same time, ties are broken such that honest nodes from now on build on what was the second withheld block. This is crucial, as it allows the adversary to reuse in the form of *equivocations* the withheld blocks 3, 4, ..., $k$ on top of the chain Genesis -> 1 -> 2 formed by the first two withheld adversarial blocks, which is now the chain adopted by honest nodes.

As an overall result of the attack so far, the adversary started with $k$ withheld blocks, has used those to displace $k$ honest blocks, and is now left with equivocating copies of $k-2$ adversarial withheld blocks that it can still reuse through equivocations. In addition, while the $k$ honest blocks were produced, the adversary probably had a few block production opportunities of its own, which get added to the pool of adversarial withheld blocks. (Note that the attack has renewed in favor of the adversary if the adversary had two new block production opportunities, making up for the two adversarial withheld blocks lost because they cannot be reused.)

The process now repeats: The adversary has a bunch withheld blocks; whenever honest nodes have built a chain of weight equal to the withheld blocks, then the adversary releases a competing sub-tree of height 2; the chain made up from the first two released withheld blocks is adopted by honest nodes, the other block production opportunities can still be reused in the future through equivocations on top of it and thus remain in the pool of withheld blocks of the adversary.

If the adversary starts out with enough withheld blocks $k$, and adversarial stake is not too small, then the adversary gains 2 block production opportunities during the production of the $k$ honest blocks that will be displaced subsequently, and the process renews (or even drifts in favor of the adversary). No honest blocks enter the canonical chain permanently.
