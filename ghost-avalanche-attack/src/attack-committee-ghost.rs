mod lottery;
mod blocks;
mod utils;
mod aux;

use crate::lottery::{LeaderSequence, Leader, Timeslot};
use crate::blocks::{GhostBlockTree, VoteWeight};
use crate::utils::{Digest};
use crate::aux::{honest_votes_to_match, create_block, honest_chain_has_started, Party};
use crate::scenario::{CONFIG_RANDOM_SEED, CONFIG_COMMITTEE_HONEST, CONFIG_COMMITTEE_ADVERSARIAL, CONFIG_ADV_HEADSTART, CONFIG_MAX_TIMESLOT};


// scenario for short attacks (for illustration purposes)
#[cfg(feature = "shortscenario")]
mod scenario {
    use crate::lottery::{Timeslot};
    use crate::blocks::{VoteWeight};

    pub const CONFIG_RANDOM_SEED: u64 = 42;
    pub const CONFIG_COMMITTEE_HONEST: VoteWeight = 80;
    pub const CONFIG_COMMITTEE_ADVERSARIAL: VoteWeight = 20;
    pub const CONFIG_ADV_HEADSTART: usize = 12;
    pub const CONFIG_MAX_TIMESLOT: Timeslot = 100;
}

// scenario for long attacks
#[cfg(not(feature = "shortscenario"))]
mod scenario {
    use crate::lottery::{Timeslot};
    use crate::blocks::{VoteWeight};

    pub const CONFIG_RANDOM_SEED: u64 = 42;
    pub const CONFIG_COMMITTEE_HONEST: VoteWeight = 80;
    pub const CONFIG_COMMITTEE_ADVERSARIAL: VoteWeight = 20;
    pub const CONFIG_ADV_HEADSTART: usize = 15;
    pub const CONFIG_MAX_TIMESLOT: Timeslot = 1000;
}


fn main() {
    const CONFIG_COMMITTEE: VoteWeight = CONFIG_COMMITTEE_HONEST + CONFIG_COMMITTEE_ADVERSARIAL;
    const CONFIG_BETA: f64 = (CONFIG_COMMITTEE_ADVERSARIAL as f64) / (CONFIG_COMMITTEE as f64);


    let mut leaderseq = LeaderSequence::new_with_adversarial_headstart(CONFIG_RANDOM_SEED, CONFIG_BETA, CONFIG_ADV_HEADSTART);
    let mut blktree = GhostBlockTree::new();

    // counting honest/adversarial block production opportunities
    let mut count_honest = 0;
    let mut count_adversarial = 0;

    // adv block production opportunities and votes that can be used to build a sub-tree to displace honest chain
    let mut adv_withheld_blocks = Vec::<(Timeslot, usize)>::new();
    let mut adv_withheld_votes = Vec::<Timeslot>::new();
    // where to build the displacing adversarial sub-tree on
    let mut adv_release_target_digest = blktree.get_genesis_digest();

    // aux variable to order blocks nicely for plotting purposes only (dot file)
    let mut aux_node_ordering = Vec::<Vec<Digest>>::new();


    for timeslot in 1.. {
        println!("t={}: {:?} leader", timeslot, leaderseq.get(timeslot));

        // ADVERSARY: rushing adversary, so perform adversarial action first

        // sanity check bookkeeping
        assert!(adv_withheld_blocks.windows(2).all(|w| w[0] <= w[1]));
        assert!(adv_withheld_votes.windows(2).all(|w| w[0] <= w[1]));

        // consider releasing withheld blocks/votes only if there is an honest chain that could be displaced
        if honest_chain_has_started(&blktree, &adv_release_target_digest) {
            // since we can equivocate on the second layer of the adversarial displacing tree,
            // there should really never be a shortage of withheld blocks ... so might
            // as well assume there are at least three entries in that list at all times
            assert!(adv_withheld_blocks.len() >= 3);

            if honest_votes_to_match(&blktree, &adv_release_target_digest) <= CONFIG_COMMITTEE_ADVERSARIAL * adv_withheld_votes.len()
                    && CONFIG_COMMITTEE_ADVERSARIAL * adv_withheld_votes.len() < honest_votes_to_match(&blktree, &adv_release_target_digest) + CONFIG_COMMITTEE_HONEST  {

                // release withheld blocks, otherwise the newly arriving honest block risks taking away our opportunity to kick out honest blocks

                let (t1, c1) = adv_withheld_blocks.remove(0);
                let (t2, c2) = adv_withheld_blocks.remove(0);
                let new_block1_digest = create_block(&mut blktree, &adv_release_target_digest, format!("{} tR={} [A]", c1, timeslot), t1);
                let new_block2_digest = create_block(&mut blktree, &new_block1_digest, format!("{}-{:05} tR={} [A]", c2, 1, timeslot), t2);

                let t3 = adv_withheld_blocks[0].0;
                let mut votes_for_level2 = 0;
                let mut votes_to_reuse = Vec::<Timeslot>::new();

                while blktree.get_tip() != new_block2_digest {
                    let t = adv_withheld_votes.remove(0);

                    if t < t1 {
                        // can't use anymore, kick out (should not happen!)
                        assert!(false);

                    } else if t1 <= t && t < t2 {
                        // can only be cast for the first level, so do that
                        blktree.add_vote(&new_block1_digest, t, CONFIG_COMMITTEE_ADVERSARIAL, Party::Adversarial as usize);

                    } else if t2 <= t && votes_for_level2 == 0 {
                        // can be cast for the second level, and no vote has been cast for the second level so far, so do that
                        blktree.add_vote(&new_block2_digest, t, CONFIG_COMMITTEE_ADVERSARIAL, Party::Adversarial as usize);
                        votes_for_level2 += 1;

                    } else { // if t2 <= t && has_voted_on_level2
                        // can be cast for the second level, and a vote has been cast there already, so add more blocks
                        let new_block2_clone_digest = create_block(&mut blktree, &new_block1_digest, format!("{}-{:05} tR={} [A]", c2, votes_for_level2+1, timeslot), t2);
                        blktree.add_vote(&new_block2_clone_digest, t, CONFIG_COMMITTEE_ADVERSARIAL, Party::Adversarial as usize);
                        votes_for_level2 += 1;

                    }
                    
                    if t3 <= t {
                        votes_to_reuse.push(t);
                    }
                }

                votes_to_reuse.append(&mut adv_withheld_votes);
                adv_withheld_votes = votes_to_reuse;

                aux_node_ordering.push(blktree.get_children_of(&adv_release_target_digest));
                aux_node_ordering.push(blktree.get_children_of(&new_block1_digest));

                assert!(blktree.get_tip() == new_block2_digest);
                adv_release_target_digest = blktree.get_tip();

                println!("{}", blktree.get_dotfile(&aux_node_ordering));

            } else if CONFIG_COMMITTEE_ADVERSARIAL * adv_withheld_votes.len() < honest_votes_to_match(&blktree, &adv_release_target_digest) {
                // we lost: an honest block is entering the canonical chain permanently ... :(((

                println!("{}", blktree.get_dotfile(&aux_node_ordering));
                panic!("Honest block entered canonical chain permanently: {}", timeslot);

            } else {
                // no need to act right now, can tolerate another honest block
                // getting added to the honest chain and consider acting then

            }
        }

        if leaderseq.get(timeslot) == Leader::Adversarial {
            // withhold this block (block production opportunity) for future release on adversarial sub-tree
            count_adversarial += 1;
            adv_withheld_blocks.push((timeslot, count_adversarial));
        }

        // withhold votes from this slot for future release on adversarial sub-tree
        adv_withheld_votes.push(timeslot);
        

        // HONEST

        if leaderseq.get(timeslot) == Leader::Honest {
            let tip_digest = blktree.get_tip();
            count_honest += 1;
            create_block(&mut blktree, &tip_digest, format!("{} [H]", count_honest), timeslot);
        }

        let tip_digest = blktree.get_tip();
        blktree.add_vote(&tip_digest, timeslot, CONFIG_COMMITTEE_HONEST, Party::Honest as usize);
        println!("Voting for {:?}", tip_digest);



        if timeslot >= CONFIG_MAX_TIMESLOT {
            break;
        }
    }


    println!("{}", blktree.get_dotfile(&aux_node_ordering));
    eprintln!("{}", blktree.get_dotfile(&aux_node_ordering));
}

