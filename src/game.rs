use wasm_bindgen::prelude::*;
use crate::{consts::N_MOVES, rotation::Rotation};


#[wasm_bindgen]
#[derive(Clone)]
pub struct Board {
    pub(crate) rotation: Rotation,  // only used with JS FFI
    pub(crate) cells: [CellValue; N_MOVES],
    pub(crate) p0_wants: Outcome,
    pub(crate) p1_wants: Outcome,
    pub(crate) turn: u8,
    pub(crate) infoset: Infoset,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CellValue { Empty, P0, P1 }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Outcome { Tie, P0Win, P1Win }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Move(pub(crate) usize);


// infosets
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct Infoset { 
    p0_private: u32, 
    p1_private: u32,
    history: u32
}

impl Infoset {
    pub fn to_key(&self, as_p0: bool) -> (u32, u32) {
        (self.history, if as_p0 { self.p0_private } else { self.p1_private })
    }
}

impl Board {
    pub fn new(p0_wants: Outcome, p1_wants: Outcome) -> Self {
        Self {
            rotation: Rotation::Straight,
            cells: [CellValue::Empty; 9],
            p0_wants, p1_wants,
            turn: 0,
            infoset: Infoset {
                history: 1,
                p0_private: p0_wants.to_smallint(),
                p1_private: p1_wants.to_smallint(),
            }
        }
    }

    pub fn possible_moves(&self) -> Vec<Move> {
        if self.turn >= N_MOVES as u8 { return vec![] }
        if self.turn == 0 { return vec![Move(0), Move(1), Move(4)]; };

        let mut out = vec![];
        for (i, cell) in self.cells.iter().enumerate() {
            // symmetries on the board

            if let CellValue::Empty = cell { 
                out.push(Move(i))
            }
        }
        return out;
    }

    pub fn next_to_move(&self) -> CellValue {
        if self.turn % 2 == 0 { CellValue::P0 } else { CellValue::P1 }
    }

    pub fn play(&mut self, m: Move) {
        assert!(m.0 < self.cells.len() && self.cells[m.0] == CellValue::Empty);
        self.cells[m.0] = self.next_to_move();
        self.turn += 1;

        self.infoset.history *= N_MOVES as u32;
        self.infoset.history += m.0 as u32;
    }

    pub fn score(&self) -> Option<(Outcome, i8, i8)> {
        let all_eq = |a, b, c, d| a == b && b == c && c == d;

        let mut outcome = None;
        for (p, possible_outcome) in [(CellValue::P0, Outcome::P0Win), (CellValue::P1, Outcome::P1Win)] {
            if 
                all_eq(p, self.cells[0], self.cells[1], self.cells[2]) || 
                all_eq(p, self.cells[3], self.cells[4], self.cells[5]) || 
                all_eq(p, self.cells[6], self.cells[7], self.cells[8]) || 

                all_eq(p, self.cells[0], self.cells[3], self.cells[6]) || 
                all_eq(p, self.cells[1], self.cells[4], self.cells[7]) || 
                all_eq(p, self.cells[2], self.cells[5], self.cells[8]) || 

                all_eq(p, self.cells[0], self.cells[4], self.cells[8]) || 
                all_eq(p, self.cells[2], self.cells[4], self.cells[6])
            {
                outcome = Some(possible_outcome);
                break;
            }
        }

        if self.turn >= N_MOVES as u8 && outcome == None {
            outcome = Some(Outcome::Tie);
        }

        if let Some(o) = outcome {
            return Some((o, (self.p0_wants == o) as i8, (self.p1_wants == o) as i8))
        }
        return None
    }
}

impl Outcome {
    fn to_smallint(self) -> u32 {
        match self {
            Outcome::Tie => 0,
            Outcome::P0Win => 1,
            Outcome::P1Win => 2,
        }
    }
}