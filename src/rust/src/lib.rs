use extendr_api::prelude::*;
// use ordered_float::OrderedFloat;
// use extendr_api::wrapper::Doubles;
use indicatif::ProgressBar;
use rand::thread_rng;
use rand::Rng;
use ndarray::{Array2};
use rayon::prelude::*;

fn sub_evaluation(assignment: &[bool], outcome: &[f64]) -> f64 {
    assignment
        .iter()
        .zip(outcome.iter())
        .filter(|(x, _)| **x)
        .map(|(_, y)| *y)
        .sum()
}

#[derive(Debug, Clone)]
struct Policy {
    assignment: Vec<bool>,
    n: usize,
    n_treat: usize,
    utility_1: Option<f64>,
    utility_2: Option<f64>,
    rank: Option<i32>,
}

#[derive(Debug)]
struct Population {
    policies: Vec<Policy>,
    n: usize,
    n_treat: usize,
    temperature: f64,
    temperature_decay: f64,
    po_1_t: Vec<f64>,
    po_1_c: Vec<f64>,
    po_2_t: Vec<f64>,
    po_2_c: Vec<f64>,
    gen: i32,
    gen_size: i32,
}

impl Population {
    fn new(po_1_t: Vec<f64>, po_1_c: Vec<f64>, po_2_t: Vec<f64>, po_2_c: Vec<f64>, n_treat: usize, temperature_decay:f64) -> Population {
        let mut pop = Population {
            policies: Vec::new(),
            n_treat: n_treat,
            n: po_1_t.len(),
            po_1_t: po_1_t,
            po_1_c: po_1_c,
            po_2_t: po_2_t,
            po_2_c: po_2_c,
            temperature: 1.0,
            temperature_decay: temperature_decay,
            gen: 0,
            gen_size: 5000,
        };

        for _ in 0..pop.gen_size {
            pop.policies.push(Policy::new(pop.n, pop.n_treat));
        }
        pop
    }

    fn evaluate(&mut self) {
        self.policies
            .par_iter_mut()
            .for_each(|policy| policy.evaluate(&self.po_1_t, &self.po_1_c, &self.po_2_t, &self.po_2_c));


        self.policies.sort_by(|a, b| {
            b.utility_1
                .unwrap()
                .partial_cmp(&a.utility_1.unwrap())
                .unwrap()
        });

        let mut rank = 0;
        while self.policies.iter().any(|x| x.rank.is_none()) && rank < 10 {
            rank += 1;
            let mut current_best_y = 0.0;
            for policy in self.policies.iter_mut() {
                if policy.rank.is_none() {
                    if current_best_y < policy.utility_2.unwrap() {
                        policy.rank = Some(rank);
                        current_best_y = policy.utility_2.unwrap();
                    }
                }
            }
        }

        for policy in self.policies.iter_mut() {
            if policy.rank.is_none() {
                policy.rank = Some(99)
            }
        }
    }

    fn next_gen(&mut self) {
        let mut next_gen: Vec<Policy> = Vec::new();
        self.gen += 1;

        self.temperature *= self.temperature_decay;
        let mut num_mutates = (self.temperature * (self.po_2_c.len() as f64)) as i32;
        num_mutates = std::cmp::max(num_mutates, 10);

        for policy in &self.policies {
            if policy.rank.unwrap() == 1 {
                let mut kid = policy.clone();
                kid.rank = None;
                kid.utility_1 = None;
                kid.utility_2 = None;

                next_gen.push(kid.clone());
                kid.mutate(num_mutates);
                next_gen.push(kid);
            }
        }

        let mut kids : Vec<Policy> = (0..((self.gen_size - (next_gen.len() as i32)) as usize))
            .into_par_iter()
            .map(|_| self.create_1_kid(num_mutates))
            .collect();

        next_gen.append(&mut kids);
        self.policies = next_gen;
    }

    fn create_1_kid(&self, num_mutates : i32) -> Policy{
        let mut rng = thread_rng();
        let i1 = rng.gen_range(0..self.gen_size) as usize;
        let i2 = rng.gen_range(0..self.gen_size) as usize;
        let i3 = rng.gen_range(0..self.gen_size) as usize;
        let i4 = rng.gen_range(0..self.gen_size) as usize;
        let c1: usize;
        let c2: usize;

        if self.policies[i1].rank.unwrap() < self.policies[i2].rank.unwrap() {
            c1 = i1
        } else {
            c1 = i2
        }

        if self.policies[i3].rank.unwrap() < self.policies[i4].rank.unwrap() {
            c2 = i3
        } else {
            c2 = i4
        }

        let mut kid = self.policies[c1].merge(&self.policies[c2]);
        kid.repair();
        kid.mutate(num_mutates);
        kid
    }
}

impl Policy {
    fn new(n: usize, n_treat: usize) -> Policy {
        let mut policy = Policy {
            assignment: vec![false; n as usize],
            n: n,
            n_treat: n_treat,
            utility_1: None,
            utility_2: None,
            rank: None,
        };

        let mut rng = thread_rng();
        let indexes = rand::seq::index::sample(&mut rng, n, n_treat);

        for index in indexes {
            policy.assignment[index] = true;
        }

        policy
    }

    fn mutate(&mut self, n_mutates: i32) {
        let mut rng = rand::thread_rng();
        for _ in 0..n_mutates {
            let i1 = rng.gen_range(0..self.n);
            let i2 = rng.gen_range(0..self.n);
            self.assignment.swap(i1, i2);
        }
    }

    fn repair(&mut self) {
        let mut current_n_treated: usize = self.assignment.iter().filter(|x| **x).count();

        let mut rng = rand::thread_rng();
        while current_n_treated != self.n_treat {
            let idx = rng.gen_range(0..self.n);

            if self.assignment[idx] && current_n_treated > self.n_treat {
                self.assignment[idx] = false;
                current_n_treated -= 1;
            } else if !self.assignment[idx] && current_n_treated < self.n_treat {
                self.assignment[idx] = true;
                current_n_treated += 1;
            }
        }
    }

    fn merge(&self, other: &Self) -> Self {
        let mut policy = Policy {
            assignment: self.assignment.clone(),
            n: self.n,
            n_treat: self.n_treat,
            utility_1: None,
            utility_2: None,
            rank: None,
        };

        for i in 0..self.n {
            if rand::random() {
                policy.assignment[i] = other.assignment[i];
            }
        }
        policy
    }

    fn evaluate(
        &mut self,
        po_1_t: &Vec<f64>,
        po_1_c: &Vec<f64>,
        po_2_t: &Vec<f64>,
        po_2_c: &Vec<f64>,
    ) {
        let mut utility_1 = sub_evaluation(&self.assignment, po_1_t);
        utility_1 += sub_evaluation(&self.assignment.iter().map(|x| !x).collect::<Vec<bool>>(), po_1_c);

        let mut utility_2 = sub_evaluation(&self.assignment, po_2_t);
        utility_2 += sub_evaluation(&self.assignment.iter().map(|x| !x).collect::<Vec<bool>>(), po_2_c);

        self.utility_1 = Some(utility_1);
        self.utility_2 = Some(utility_2);
    }
}

#[extendr]
fn gen_opt(po_1_t: &[f64], po_1_c: &[f64], po_2_t: &[f64], po_2_c: &[f64], n_treat: i32, n_iter: u64,temperature_decay : f64) -> Result<Robj> {

    let mut pop = Population::new(
        po_1_t.to_vec(),
        po_1_c.to_vec(),
        po_2_t.to_vec(),
        po_2_c.to_vec(),
        n_treat as usize,
        temperature_decay,
    );

    let bar = ProgressBar::new(n_iter);
    for _ in 0..n_iter {
        pop.evaluate();
        pop.next_gen();
        bar.inc(1);
    }

    pop.evaluate();
    let mut cols = 0;
    for policy in &pop.policies {
        if policy.rank == Some(1) {
            cols +=1
        }
    }

    let mut i = 0;
    let mut out_arr : Array2<bool> = Array2::from_elem((pop.policies[0].assignment.len(), cols-1),false);
    for mut col in out_arr.axis_iter_mut(Axis(1)) {
        loop {
            i += 1;
            if pop.policies[i].rank == Some(1) {
                 col.assign(&Array::from_vec(pop.policies[i].assignment.clone()));
                 break
            }
        }
    }


    let array = Robj::try_from(&out_arr).unwrap();

    Ok(array)

}

// Macro to generate exports.
// This ensures exported functions are registered with R.
// See corresponding C code in `entrypoint.c`.
extendr_module! {
    mod rustopt;
    fn gen_opt;
}
