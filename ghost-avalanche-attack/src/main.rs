mod lottery;
mod blocks;
mod utils;

use crate::lottery::{LeaderSequence, Leader, Timeslot};
use crate::blocks::{GhostBlockTree, Block, BlockPayload, VoteWeight};
use crate::utils::{Digest, Digestable};


// utility to create a block, attach it to the block tree, and cast votes for it
fn aux_create_block(tree: &mut GhostBlockTree, parent: &Digest, payload: String, timeslot: Timeslot, voteweight: VoteWeight) -> Digest {
    let new_block = Block::new(parent.clone(), BlockPayload(payload.to_string()), timeslot);
    let new_block_digest = new_block.digest();
    tree.add_block(new_block);
    tree.add_vote(&new_block_digest, timeslot, voteweight);

    new_block_digest
}


fn main() {
    const CONFIG_RANDOM_SEED: u64 = 42;


    // SCENARIOS for short attacks (for illustration purposes)

    // attack on PoS GHOST (attack-pos-ghost.png)
    // (each block has unit weight)
    const CONFIG_BETA: f64 = 0.3;
    const CONFIG_ADV_HEADSTART: usize = 5;
    const CONFIG_VOTES_ADV_BLOCKS: VoteWeight = 1;
    const CONFIG_VOTES_HON_BLOCKS: VoteWeight = 1;
    const CONFIG_MAX_TIMESLOT: Timeslot = 100;

    // // attack on PoS Committee-GHOST (attack-committee-ghost.png)
    // // (honest and adversarial blocks have vote weight proportional to stake)
    // const CONFIG_BETA: f64 = 0.2;
    // const CONFIG_ADV_HEADSTART: usize = 25;
    // const CONFIG_VOTES_ADV_BLOCKS: VoteWeight = 20;
    // const CONFIG_VOTES_HON_BLOCKS: VoteWeight = 80;
    // const CONFIG_MAX_TIMESLOT: Timeslot = 100;


    // SCENARIOS for long attacks

    // // attack on PoS GHOST
    // // (each block has unit weight)
    // const CONFIG_BETA: f64 = 0.3;
    // const CONFIG_ADV_HEADSTART: usize = 4;
    // const CONFIG_VOTES_ADV_BLOCKS: VoteWeight = 1;
    // const CONFIG_VOTES_HON_BLOCKS: VoteWeight = 1;
    // const CONFIG_MAX_TIMESLOT: Timeslot = 1000;

    // // attack on PoS Committee-GHOST
    // // (honest and adversarial blocks have vote weight proportional to stake)
    // const CONFIG_BETA: f64 = 0.2;
    // const CONFIG_ADV_HEADSTART: usize = 50;
    // const CONFIG_VOTES_ADV_BLOCKS: VoteWeight = 20;
    // const CONFIG_VOTES_HON_BLOCKS: VoteWeight = 80;
    // const CONFIG_MAX_TIMESLOT: Timeslot = 1000;


    let mut leaderseq = LeaderSequence::new_with_adversarial_headstart(CONFIG_RANDOM_SEED, CONFIG_BETA, CONFIG_ADV_HEADSTART);
    let mut blktree = GhostBlockTree::new();

    // counting honest/adversarial block production opportunities
    let mut count_honest = 0;
    let mut count_adversarial = 0;

    // adv block production opportunities that can be used to build a sub-tree to displace honest chain
    let mut adv_withheld_blks = Vec::<(Timeslot, usize)>::new();
    // where to build the displacing adversarial sub-tree on
    let mut adv_release_target_digest = blktree.get_genesis_digest();
    // how many honest blocks are on the chain that needs displacing
    let mut adv_honest_need_to_match = 0;

    // aux variable to order blocks nicely for plotting purposes only (dot file)
    let mut aux_node_ordering = Vec::<Vec<Digest>>::new();


    for timeslot in 1.. {
        println!("t={}: {:?} leader", timeslot, leaderseq.get(timeslot));

        // ADVERSARY: rushing adversary, so perform adversarial action first
        match leaderseq.get(timeslot) {
            Leader::Honest => {
                if CONFIG_VOTES_HON_BLOCKS * adv_honest_need_to_match <= CONFIG_VOTES_ADV_BLOCKS * adv_withheld_blks.len()
                        && CONFIG_VOTES_HON_BLOCKS * (adv_honest_need_to_match + 1) > CONFIG_VOTES_ADV_BLOCKS * adv_withheld_blks.len() {

                    // release withheld blocks, otherwise the newly arriving honest block risks taking away our opportunity to kick out honest blocks

                    assert!(adv_withheld_blks.len() >= 1);

                    if adv_withheld_blks.len() == 1 {
                        // only one withheld block :(

                        let (t, c) = adv_withheld_blks.pop().unwrap();
                        let new_block1_digest = aux_create_block(&mut blktree, &adv_release_target_digest, format!("{} [A]", c), t, CONFIG_VOTES_ADV_BLOCKS);

                        aux_node_ordering.push(blktree.get_children_of(&adv_release_target_digest));

                        assert!(blktree.get_tip() == new_block1_digest);
                        adv_withheld_blks = Vec::new();
                        adv_release_target_digest = blktree.get_tip();
                        adv_honest_need_to_match = 0;

                    } else { // if adv_withheld_blks.len() > 1
                        // yay, many withheld blocks! :)

                        let (t1, c1) = adv_withheld_blks.remove(0);
                        let (t2, c2) = adv_withheld_blks.remove(0);
                        let new_block1_digest = aux_create_block(&mut blktree, &adv_release_target_digest, format!("{} [A]", c1), t1, CONFIG_VOTES_ADV_BLOCKS);
                        let new_block2_digest = aux_create_block(&mut blktree, &new_block1_digest, format!("{} [A]", c2), t2, CONFIG_VOTES_ADV_BLOCKS);

                        for j in 0..adv_withheld_blks.len() {
                            aux_create_block(&mut blktree, &new_block1_digest, format!("{} [A]", adv_withheld_blks[j].1), adv_withheld_blks[j].0, CONFIG_VOTES_ADV_BLOCKS);

                            if blktree.get_tip() == new_block2_digest {
                                break;
                            }
                        }

                        aux_node_ordering.push(blktree.get_children_of(&adv_release_target_digest));
                        aux_node_ordering.push(blktree.get_children_of(&new_block1_digest));

                        assert!(blktree.get_tip() == new_block2_digest);
                        adv_release_target_digest = blktree.get_tip();
                        adv_honest_need_to_match = 0;

                    }

                    println!("{}", blktree.get_dotfile(&aux_node_ordering));

                } else if CONFIG_VOTES_HON_BLOCKS * adv_honest_need_to_match > CONFIG_VOTES_ADV_BLOCKS * adv_withheld_blks.len() {
                    // we lost: an honest block is entering the canonical chain permanently ... :(((

                    // if we wanted to restart the attack ...
                    // adv_withheld_blks = Vec::new();
                    // adv_release_target_digest = blktree.get_tip();
                    // adv_honest_need_to_match = 0;

                    println!("{}", blktree.get_dotfile(&aux_node_ordering));

                    panic!("Honest block entered canonical chain permanently: {}", timeslot);

                } else {
                    // no need to act right now, can tolerate another honest block
                    // getting added to the honest chain and consider acting then

                }

                adv_honest_need_to_match += 1;   // a new honest block will be attached to the honest chain in this timeslot!

            },
            Leader::Adversarial => {
                // withhold this block (block production opportunity) for future release on adversarial sub-tree
                count_adversarial += 1;
                adv_withheld_blks.push((timeslot, count_adversarial));

            },
        }
        
        // HONEST
        if leaderseq.get(timeslot) == Leader::Honest {
            let tip_digest = blktree.get_tip();
            count_honest += 1;
            aux_create_block(&mut blktree, &tip_digest, format!("{} [H]", count_honest), timeslot, CONFIG_VOTES_HON_BLOCKS);
        }


        if timeslot >= CONFIG_MAX_TIMESLOT {
            break;
        }
    }
}

