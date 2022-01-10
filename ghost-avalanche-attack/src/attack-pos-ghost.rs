mod lottery;
mod blocks;
mod utils;
mod aux;

use crate::lottery::{LeaderSequence, Leader, Timeslot};
use crate::blocks::{GhostBlockTree};
use crate::utils::{Digest};
use crate::aux::{honest_votes_to_match, create_and_vote_for_block, Party};
use crate::scenario::{CONFIG_RANDOM_SEED, CONFIG_BETA, CONFIG_ADV_HEADSTART, CONFIG_MAX_TIMESLOT};


// scenario for short attacks (for illustration purposes)
#[cfg(feature = "shortscenario")]
mod scenario {
    use crate::lottery::{Timeslot};
    
    pub const CONFIG_RANDOM_SEED: u64 = 42;
    pub const CONFIG_BETA: f64 = 0.3;
    pub const CONFIG_ADV_HEADSTART: usize = 4;
    pub const CONFIG_MAX_TIMESLOT: Timeslot = 100;
}

// scenario for long attacks
#[cfg(not(feature = "shortscenario"))]
mod scenario {
    use crate::lottery::{Timeslot};
    
    pub const CONFIG_RANDOM_SEED: u64 = 42;
    pub const CONFIG_BETA: f64 = 0.3;
    pub const CONFIG_ADV_HEADSTART: usize = 4;
    pub const CONFIG_MAX_TIMESLOT: Timeslot = 1000;
}


fn main() {
    let mut leaderseq = LeaderSequence::new_with_adversarial_headstart(CONFIG_RANDOM_SEED, CONFIG_BETA, CONFIG_ADV_HEADSTART);
    let mut blktree = GhostBlockTree::new();

    // counting honest/adversarial block production opportunities
    let mut count_honest = 0;
    let mut count_adversarial = 0;

    // adv block production opportunities that can be used to build a sub-tree to displace honest chain
    let mut adv_withheld_blks = Vec::<(Timeslot, usize)>::new();
    // where to build the displacing adversarial sub-tree on
    let mut adv_release_target_digest = blktree.get_genesis_digest();

    // aux variable to order blocks nicely for plotting purposes only (dot file)
    let mut aux_node_ordering = Vec::<Vec<Digest>>::new();


    for timeslot in 1.. {
        println!("t={}: {:?} leader", timeslot, leaderseq.get(timeslot));


        // ADVERSARY: rushing adversary, so perform adversarial action first

        let honest_vote_gain_this_slot = if leaderseq.get(timeslot) == Leader::Honest { 1 } else { 0 };
        if honest_votes_to_match(&blktree, &adv_release_target_digest) <= adv_withheld_blks.len()
                && adv_withheld_blks.len() < honest_votes_to_match(&blktree, &adv_release_target_digest) + honest_vote_gain_this_slot {

            // release withheld blocks, otherwise the newly arriving honest block risks taking away our opportunity to kick out honest blocks

            assert!(adv_withheld_blks.len() >= 1);

            if adv_withheld_blks.len() == 1 {
                // only one withheld block :(

                let (t, c) = adv_withheld_blks.pop().unwrap();
                let new_block1_digest = create_and_vote_for_block(&mut blktree, &adv_release_target_digest, format!("{} [A]", c), t, 1, Party::Adversarial);

                aux_node_ordering.push(blktree.get_children_of(&adv_release_target_digest));

                assert!(blktree.get_tip() == new_block1_digest);
                adv_withheld_blks = Vec::new();
                adv_release_target_digest = blktree.get_tip();

            } else { // if adv_withheld_blks.len() > 1
                // yay, many withheld blocks! :)

                let (t1, c1) = adv_withheld_blks.remove(0);
                let (t2, c2) = adv_withheld_blks.remove(0);
                let new_block1_digest = create_and_vote_for_block(&mut blktree, &adv_release_target_digest, format!("{} [A]", c1), t1, 1, Party::Adversarial);
                let new_block2_digest = create_and_vote_for_block(&mut blktree, &new_block1_digest, format!("{} [A]", c2), t2, 1, Party::Adversarial);

                for j in 0..adv_withheld_blks.len() {
                    create_and_vote_for_block(&mut blktree, &new_block1_digest, format!("{} [A]", adv_withheld_blks[j].1), adv_withheld_blks[j].0, 1, Party::Adversarial);

                    if blktree.get_tip() == new_block2_digest {
                        break;
                    }
                }

                aux_node_ordering.push(blktree.get_children_of(&adv_release_target_digest));
                aux_node_ordering.push(blktree.get_children_of(&new_block1_digest));

                assert!(blktree.get_tip() == new_block2_digest);
                adv_release_target_digest = blktree.get_tip();

            }

            println!("{}", blktree.get_dotfile(&aux_node_ordering));

        } else if adv_withheld_blks.len() < honest_votes_to_match(&blktree, &adv_release_target_digest) {
            // we lost: an honest block is entering the canonical chain permanently ... :(((

            // if we wanted to restart the attack ...
            // adv_withheld_blks = Vec::new();
            // adv_release_target_digest = blktree.get_tip();

            println!("{}", blktree.get_dotfile(&aux_node_ordering));
            panic!("Honest block entered canonical chain permanently: {}", timeslot);

        } else {
            // no need to act right now, can tolerate another honest block
            // getting added to the honest chain and consider acting then

        }

        if leaderseq.get(timeslot) == Leader::Adversarial {
            // withhold this block (block production opportunity) for future release on adversarial sub-tree
            count_adversarial += 1;
            adv_withheld_blks.push((timeslot, count_adversarial));
        }
        

        // HONEST

        if leaderseq.get(timeslot) == Leader::Honest {
            let tip_digest = blktree.get_tip();
            count_honest += 1;
            create_and_vote_for_block(&mut blktree, &tip_digest, format!("{} [H]", count_honest), timeslot, 1, Party::Honest);
        }



        if timeslot >= CONFIG_MAX_TIMESLOT {
            break;
        }
    }


    println!("{}", blktree.get_dotfile(&aux_node_ordering));
    eprintln!("{}", blktree.get_dotfile(&aux_node_ordering));
}

