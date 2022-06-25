use rand::{thread_rng, prelude::SliceRandom};
use wasm_bindgen::prelude::*;

use crate::{game::{Board, Outcome, CellValue, Move}, consts::N_MOVES, strategy::STRATEGY, utils::set_panic_hook, rotation::Rotation};

#[wasm_bindgen]
#[allow(dead_code)]
pub fn init() {
    set_panic_hook()
}

#[wasm_bindgen]
impl Board {
    pub fn js_start_random() -> Board {
        Board::new(
            *[Outcome::P0Win, Outcome::P1Win, Outcome::Tie].choose(&mut thread_rng()).unwrap(),
            *[Outcome::P0Win, Outcome::P1Win, Outcome::Tie].choose(&mut thread_rng()).unwrap(),
        )
    }

    fn calculate_advice(&self) -> [f32; 9] {
        if self.turn == 0 {
            // turn 0 advice
            // average out advice across all rotations
            let base_advice = STRATEGY.with(|s| s.distribution(self));
            let advices = [
                base_advice,
                Rotation::Left.rotate_matrix(base_advice),
                Rotation::Double.rotate_matrix(base_advice),
                Rotation::Right.rotate_matrix(base_advice)
            ];

            let mut avg_advice = [0.0; N_MOVES];
            for i in 0..4 {
                for m in 0..N_MOVES {
                    avg_advice[m] += advices[i][m]/4.0;
                }
            }

            avg_advice
        } else {
            // hack to check for moves that accomplish current player's wincon
            // if so, that move must take place
            let mut immediate_winning_moves = vec![];
            for m in 0..N_MOVES as u8 {
                let mut b2 = self.clone();
                b2.js_play(m);
                if let Some((_, p0, p1)) = b2.score() {
                    let my_score = match self.next_to_move() {
                        CellValue::P0 => p0,
                        CellValue::P1 => p1,
                        _ => 0,
                    };
                    if my_score > 0 { 
                        immediate_winning_moves.push(m);
                    }
                }
            }

            let n_winning_moves = immediate_winning_moves.len();
            if n_winning_moves > 0 {
                let mut strat = [0.0; N_MOVES];
                for i in immediate_winning_moves {
                    strat[i as usize] = 1.0/n_winning_moves as f32;
                }
                return strat
            }


            self.rotation.rotate_matrix(STRATEGY.with(|s| s.distribution(self)))
        }
    }

    pub fn js_view(&self) -> View {
        let board = self.rotation.rotate_matrix(self.cells.map(|c| 
            match c {
                CellValue::Empty => 255,
                CellValue::P0 => 0,
                CellValue::P1 => 1,
            }
        ));

        let (outcome, util_p0, util_p1) = if let Some((outcome, util_p0, util_p1)) = self.score() {
            (match outcome { Outcome::P0Win => 0, Outcome::P1Win => 1, Outcome::Tie => 2 }, util_p0 as i8, util_p1 as i8)
        } else {
            (255, 0, 0)
        };

        let advice = 
            if outcome != 255 {  
                // if game is over
                [0.0; N_MOVES]
            } else {
                self.calculate_advice()
            };

        let wants_p0 = match self.p0_wants {
            Outcome::Tie => 2,
            Outcome::P0Win => 0,
            Outcome::P1Win => 1,
        };

        let wants_p1 = match self.p1_wants {
            Outcome::Tie => 2,
            Outcome::P0Win => 0,
            Outcome::P1Win => 1,
        };

        View { 
            player_turn: (self.turn % 2) as u8,
            board,
            advice,
            outcome, util_p0, util_p1, wants_p0, wants_p1,
        }
    }

    pub fn js_play(&mut self, m: u8) {
        if self.turn == 0 {
            // all moves are in principle possible
            if m == 0 { self.play(Move(0)); }
            else if m == 1 { self.play(Move(1)); }
            else if m == 2 { 
                self.rotation = Rotation::Left;
                self.play(Move(0)); 
            }
            else if m == 5 { 
                self.rotation = Rotation::Left;
                self.play(Move(1)); 
            }
            else if m == 8 { 
                self.rotation = Rotation::Double;
                self.play(Move(0)); 
            }
            else if m == 7 { 
                self.rotation = Rotation::Double;
                self.play(Move(1)); 
            }
            else if m == 6 { 
                self.rotation = Rotation::Right;
                self.play(Move(0)); 
            }
            else if m == 3 { 
                self.rotation = Rotation::Right;
                self.play(Move(1)); 
            }
            else if m == 4 { 
                self.rotation = *[
                    Rotation::Straight,
                    Rotation::Right,
                    Rotation::Double,
                    Rotation::Left,
                ].choose(&mut thread_rng()).unwrap();
                self.play(Move(4));
            } else {
                panic!("invalid move: {}", m)
            }
            return
        }

        let m = self.rotation.derotate_index(m);
        

        // explicitly handle errors by doing nothing
        if self.turn as usize >= N_MOVES { return; }
        if m as usize >= N_MOVES { return; }
        if self.cells[m as usize] != CellValue::Empty { return; }
        if self.score().is_some() { return; }
        if !self.possible_moves().contains(&Move(m as usize)) { return; }

        self.play(Move(m as usize))
    }
}

#[wasm_bindgen]
pub struct View {
    pub player_turn: u8, // 0 for p0, 1 for p1
    board: [u8; N_MOVES], // 255 for empty, 0 for p0, 1 for p1
    advice: [f32; N_MOVES], // 255 for empty, 0 for p0, 1 for p1

    // 0 for p0 wins, 1 for p1 wins, 2 for a tie, 255 otherwise
    pub outcome: u8,
    pub util_p0: i8,
    pub util_p1: i8,
    pub wants_p0: u8, // 0 for p0, 1 for p1, 2 for a tie
    pub wants_p1: u8, // 0 for p0, 1 for p1, 2 for a tie
}

#[wasm_bindgen]
impl View {
    pub fn get_cell(&self, i: usize) -> u8 {
        self.board.get(i).cloned().unwrap_or_else(|| 255)
    }

    pub fn get_advice(&self, i: usize) -> f32 {
        self.advice.get(i).cloned().unwrap_or_else(|| 0.0)
    }
}