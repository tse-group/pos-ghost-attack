use rand::{Rng,SeedableRng};


pub type Timeslot = usize;


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Leader {
    Honest,
    Adversarial,
}


#[derive(Debug, Clone)]
pub struct LeaderSequence<Rng> {
    rng: Rng,
    beta: f64,
    seq: Vec<Leader>,
}

impl LeaderSequence<rand::rngs::StdRng> {
    pub fn new(rng_seed: u64, beta: f64, seq: Vec<Leader>) -> Self {
        let rng = rand::rngs::StdRng::seed_from_u64(rng_seed);

        Self { rng: rng, beta: beta, seq: seq }
    }

    pub fn new_with_adversarial_headstart(rng_seed: u64, beta: f64, k: usize) -> Self {
        let mut seq_init = Vec::new();

        for _i in 0..k {
            seq_init.push(Leader::Adversarial);
        }

        Self::new(rng_seed, beta, seq_init)
    }

    pub fn get(&mut self, idx: Timeslot) -> Leader {
        assert!(idx > 0, "Timeslots are positive");

        while self.seq.len() < idx {
            if self.rng.gen::<f64>() < self.beta {
                self.seq.push(Leader::Adversarial);
            } else {
                self.seq.push(Leader::Honest);
            }
        }

        self.seq[idx-1]
    }
}

