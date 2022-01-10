use crate::lottery::{Timeslot};
use crate::blocks::{GhostBlockTree, Block, BlockPayload, VoteWeight};
use crate::utils::{Digest, Digestable};


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Party {
    Honest,
    Adversarial,
}

// utility to create a block and attach it to the block tree
pub fn create_block(tree: &mut GhostBlockTree, parent: &Digest, payload: String, timeslot: Timeslot) -> Digest {
    let new_block = Block::new(parent.clone(), BlockPayload(payload.to_string()), timeslot);
    let new_block_digest = new_block.digest();
    tree.add_block(new_block);

    new_block_digest
}

// utility to create a block, attach it to the block tree, and cast votes for it
#[allow(dead_code)]
pub fn create_and_vote_for_block(tree: &mut GhostBlockTree, parent: &Digest, payload: String, timeslot: Timeslot, voteweight: VoteWeight, party: Party) -> Digest {
    let new_block_digest = create_block(tree, parent, payload, timeslot);
    tree.add_vote(&new_block_digest, timeslot, voteweight, party as usize);

    new_block_digest
}

// determines how many votes the adversary has to match
pub fn honest_votes_to_match(tree: &GhostBlockTree, d: &Digest) -> VoteWeight {
    let children = tree.get_children_of(&d);
    assert!(children.len() <= 1);

    if children.len() == 0 { 0 } else { tree.get_votetally_for(&children[0]) }
}

// determines whether an honest chain has been started
#[allow(dead_code)]
pub fn honest_chain_has_started(tree: &GhostBlockTree, d: &Digest) -> bool {
    let children = tree.get_children_of(&d);

    children.len() > 0
}
