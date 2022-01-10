use crate::lottery::Timeslot;
use crate::utils::{Digest, Digestable};

use serde::{Serialize};
use ed25519_dalek::{Sha512, Digest as _};
use std::convert::{TryFrom, TryInto};
use std::collections::{HashMap, HashSet};
use itertools::Itertools;


#[derive(Debug, Default, Clone, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct BlockPayload(pub String);


#[derive(Debug, Default, Clone, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Block {
    parent: Option<Digest>,
    payload: BlockPayload,
    slot: Timeslot,
}

impl Block {
    pub fn new(parent: Digest, payload: BlockPayload, slot: Timeslot) -> Self {
        Self { parent: Some(parent), payload, slot }
    }

    pub fn genesis() -> Self {
        let mut blk = Block::default();
        blk.payload.0 = "Genesis".to_string();
        blk
    }

    pub fn _aux_is_adversarial(&self) -> bool {
        self.payload.0.ends_with("[A]")
    }
}

impl TryFrom<&Block> for Vec<u8> {
    type Error = Box<bincode::ErrorKind>;

    fn try_from(value: &Block) -> Result<Self, Self::Error> {
        bincode::serialize(&value)
    }
}

impl Digestable for Block {
    fn digest(&self) -> Digest {
        let self_binary: Vec<u8> = self.try_into().unwrap();
        let mut hasher = Sha512::new();
        hasher.update(self_binary);
        Digest(hasher.finalize().as_slice()[..32].try_into().unwrap())
    }
}


pub type VoteWeight = usize;


#[derive(Debug, Serialize, Default)]
pub struct GhostBlockTree {
    blocks: HashMap<Digest, Block>,
    children: HashMap<Digest, Vec<Digest>>,
    votetally: HashMap<Digest, VoteWeight>,
    votestallied: HashMap<Digest, HashSet<(Timeslot, usize)>>,
    debug_voteweights: HashMap<(Digest, Timeslot, usize), VoteWeight>,
    genesis_digest: Digest,
}

impl GhostBlockTree {
    pub fn new() -> Self {
        let mut blocks = HashMap::new();
        let mut children = HashMap::new();
        let mut votetally = HashMap::new();
        let mut votestallied = HashMap::new();
        let debug_voteweights = HashMap::new();

        let blk_genesis = Block::genesis();
        let blk_genesis_digest = blk_genesis.digest();

        blocks.insert(blk_genesis_digest.clone(), blk_genesis);
        children.insert(blk_genesis_digest.clone(), Vec::new());
        votetally.insert(blk_genesis_digest.clone(), 0);
        votestallied.insert(blk_genesis_digest.clone(), HashSet::new());

        Self { blocks, children, votetally, votestallied, debug_voteweights, genesis_digest: blk_genesis_digest }
    }

    pub fn get_tip(&self) -> Digest {
        let mut b0_digest = &self.genesis_digest;

        while self.children[b0_digest].len() > 0 {
            let mut max_votetally = None;
            let mut max_votetally_blk_digest: &Digest = &self.genesis_digest;

            for b_digest in &self.children[b0_digest] {
                // assert!(self.votetally[b_digest] >= 0, "Vote tallies expected to be non-negative");

                let b1 = self.blocks.get(b_digest).expect("Block expected to be in GhostBlockTree");
                let b2 = self.blocks.get(max_votetally_blk_digest).expect("Block expected to be in GhostBlockTree");

                if (max_votetally == None) || 
                    (self.votetally[b_digest] > max_votetally.unwrap()) || 
                    (self.votetally[b_digest] == max_votetally.unwrap() && 
                        b1._aux_is_adversarial() > b2._aux_is_adversarial()) ||
                    (self.votetally[b_digest] == max_votetally.unwrap() && 
                        b1._aux_is_adversarial() == b2._aux_is_adversarial() &&
                        b1.slot < b2.slot) ||
                    (self.votetally[b_digest] == max_votetally.unwrap() && 
                        b1._aux_is_adversarial() == b2._aux_is_adversarial() &&
                        b1.slot == b2.slot &&
                        b1.payload < b2.payload) {
                    max_votetally = Some(self.votetally[b_digest]);
                    max_votetally_blk_digest = &b_digest;
                }
            }

            b0_digest = max_votetally_blk_digest;
        }

        b0_digest.clone()
    }

    pub fn get_genesis_digest(&self) -> Digest {
        self.genesis_digest.clone()
    }

    pub fn get_children_of(&self, d: &Digest) -> Vec<Digest> {
        self.children.get(d).expect("Digest expected to be in GhostBlockTree").clone()
    }

    pub fn get_votetally_for(&self, d: &Digest) -> VoteWeight {
        self.votetally.get(d).expect("Digest expected to be in GhostBlockTree").clone()
    }

    pub fn add_block(&mut self, blk: Block) {
        let blk_digest = blk.digest();
        let parent_blk_digest = blk.parent.clone().expect("Parent expected to be Some(...)");

        self.blocks.insert(blk_digest.clone(), blk);
        self.children.insert(blk_digest.clone(), Vec::new());
        self.votetally.insert(blk_digest.clone(), 0);
        self.votestallied.insert(blk_digest.clone(), HashSet::new());

        self.children.get_mut(&parent_blk_digest)
            .expect("Parent expected to be in GhostBlockTree")
            .push(blk_digest);
    }

    pub fn add_vote(&mut self, d: &Digest, t: Timeslot, w: VoteWeight, party: usize) {
        assert!(self.blocks.contains_key(&d), "Digest expected to be in GhostBlockTree");
        
        let mut b = Some(d);

        while b != None {
            let b_digest = b.expect("Digest expected to be != None");
            
            let blk = &self.blocks[b_digest];
            assert!(blk.slot <= t, "Cannot cast votes on future blocks");

            if self.votestallied[b_digest].contains(&(t, party)) {
                // a vote has been tallied for this block and time slot already;
                // since we do not handle updating vote tallies, make sure the
                // current vote adding request is consistent (ie, matches the
                // weight of the previous request); no new votes added
                assert!(self.debug_voteweights[&(b_digest.clone(), t, party)] == w, "Updating/overriding previous votes not supported");
            } else {
                // otherwise add votes and keep track of what votes were tallied
                *self.votetally.get_mut(b_digest).unwrap() += w;
                self.votestallied.get_mut(b_digest).unwrap().insert((t, party));
                self.debug_voteweights.insert((b_digest.clone(), t, party), w);
            }

            b = blk.parent.as_ref();
        }
    }

    pub fn get_dotfile(&self, node_ordering: &Vec<Vec<Digest>>) -> String {
        let mut v: String = "digraph G {\n  rankdir=BT;\n  style=filled;\n  color=lightgrey;\n  node [shape=box,style=filled,color=white];\n".to_string();
        v = format!("{}\n", v);

        for blk in self.blocks.values().sorted() {
            let color = if blk._aux_is_adversarial() { "red" } else { "green" };
            v = format!(
                    "{}  blk_{} [label=\"{}\\nt={}: {}\\n{} votes\", color=\"{}\"];\n",
                    v,
                    blk.digest(),
                    format!("{}", blk.digest()).get(0..10).unwrap(),
                    blk.slot,
                    blk.payload.0.replace("\"", ""),
                    self.votetally.get(&blk.digest()).unwrap(),
                    color,
                );
        }
        v = format!("{}\n", v);

        for blk in self.blocks.values().sorted() {
            if blk.parent.clone() != None {
                v = format!("{}  blk_{} -> blk_{};\n", v, blk.digest(), blk.parent.clone().unwrap());
            }
        }

        for ordering in node_ordering {
            v = format!("{}\n", v);
            v = format!("{}  {{ rank = same; rankdir = LR; edge [style=invis];\n", v);
            v = format!("{}    ", v);
            for (i, d) in ordering.iter().enumerate() {
                if i > 0 {
                    v = format!("{} -> ", v);
                }
                v = format!("{}blk_{}", v, d);
            }
            v = format!("{}; }}\n", v);
        }
        v = format!("{}}}\n", v);

        v
    }
}

