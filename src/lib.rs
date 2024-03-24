// An attribute to hide warnings for unused code.
#![allow(dead_code)]

use std::collections::HashMap;
use std::iter;

const COMPLETE_MASK: u64 = 0xFFFFFFFFFFFFFFFE;
const POINTS_PER_ROUND: [u8; 7] = [0, 1, 2, 3, 5, 8, 13];
const SEED_ORDER: [u8; 16] = [1, 16, 8, 9, 5, 12, 4, 13, 6, 11, 3, 14, 7, 10, 2, 15];

fn seed_for_slot(slot: u8) -> u8 {
    SEED_ORDER[slot as usize % 16]
}

fn round_num_for_slot(slot: u8) -> u8 {
    let depth = f64::from(slot).log2().floor() as u8 + 1;
    7 - depth
}

trait Decisions {
    fn decisions(&self) -> u64;
    fn mask(&self) -> u64;

    fn decision_team_slots(&self) -> [Option<u8>; 64] {
        let mut res: [Option<u8>; 64] = [None; 64];
        for i in (1..=63).rev() {
            let current_position: u64 = 1 << i;
            if (current_position & self.mask()) != 0 {
                let decision = if (self.decisions() & current_position) == 0 {
                    0
                } else {
                    1
                };
                let position: u8 = (i * 2) + decision;
                res[i as usize] = if i >= 32 {
                    Some(position)
                } else {
                    res[position as usize]
                };
            }
        }

        return res;
    }
}
#[derive(Debug)]
struct Tournament {
    decisions: u64,
    mask: u64,
}

impl Decisions for Tournament {
    fn decisions(&self) -> u64 {
        return self.decisions;
    }

    fn mask(&self) -> u64 {
        return self.mask;
    }
}

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Eq, Hash)]
struct Bracket {
    decisions: u64,
}

impl Decisions for Bracket {
    fn decisions(&self) -> u64 {
        return self.decisions;
    }

    fn mask(&self) -> u64 {
        return COMPLETE_MASK;
    }
}

impl Bracket {
    fn points_for_decisions(&self, tournament_team_slots: &[Option<u8>]) -> usize {
        let bracket_team_slots = self.decision_team_slots();
        tournament_team_slots
            .into_iter()
            .enumerate()
            .fold(0, |acc, (i, t)| {
                if let Some(t) = *t {
                    if let Some(b) = bracket_team_slots[i] {
                        if t == b {
                            let team_seed = seed_for_slot(b);
                            let round_number = round_num_for_slot(i as u8);

                            return acc
                                + POINTS_PER_ROUND[round_number as usize] as usize
                                + team_seed as usize;
                        }
                    }
                }
                return acc;
            })
    }
}

struct BestFinishes {
    possible_finishes: HashMap<Bracket, usize>,
}

impl BestFinishes {
    fn new() -> BestFinishes {
        BestFinishes {
            possible_finishes: HashMap::new(),
        }
    }

    fn calc(brackets: &[Bracket], tournament_team_slots: &mut [Option<u8>]) -> BestFinishes {
        let mut best_finishes = BestFinishes::new();

        let no_decision_idx = tournament_team_slots.into_iter().rposition(|x| x.is_none());

        if no_decision_idx.is_some() && no_decision_idx != Some(0) {
            let no_decision_idx = no_decision_idx.unwrap();

            tournament_team_slots[no_decision_idx] = tournament_team_slots[no_decision_idx * 2]; // decision 0
            let child_results = BestFinishes::calc(brackets, tournament_team_slots);
            best_finishes.merge(child_results);

            tournament_team_slots[no_decision_idx] =
                tournament_team_slots[(no_decision_idx * 2) + 1]; //decision 1
            let child_results = BestFinishes::calc(brackets, tournament_team_slots);
            best_finishes.merge(child_results);

            tournament_team_slots[no_decision_idx] = None;
        } else {
            let mut tuples: Vec<(&Bracket, usize)> = brackets
                .iter()
                .map(|b| (b, b.points_for_decisions(tournament_team_slots)))
                .collect();

            tuples.sort_by(|(_, r1), (_, r2)| r2.cmp(r1));

            let mut rank = 0;
            for (i, (b, _)) in tuples.iter().enumerate() {
                if i > 0 && tuples[i - 1].1 != tuples[i].1 {
                    rank = i;
                }
                if rank > 4 {
                    //only take top-5 ranking
                    break;
                }
                best_finishes.possible_finishes.insert(**b, rank);
            }
        }
        best_finishes
    }


    fn rankings(&self) -> Vec<Vec<&Bracket>> {
        let mut ret: Vec<Vec<&Bracket>> = iter::repeat_with(|| vec![]).take(5).collect();
        self.possible_finishes.iter().for_each(|(b, rank)| {
            ret[*rank].push(b);
        });
        ret
    }

    fn merge(&mut self, other: BestFinishes) {
        other.possible_finishes.into_iter().for_each(|(b, rank)| {
            let current_rank = self.possible_finishes.get(&b);
            if current_rank.is_none() || *current_rank.unwrap() > rank {
                self.possible_finishes.insert(b, rank);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(true, true);
    }
}
