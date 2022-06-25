use std::{collections::HashMap};

use crate::{consts::N_MOVES, game::{Move, Board}};

const STRATEGY_DATA: &[u8; 322338] = include_bytes!("strategy.dat");
const VAL_EXPANSION: [f32; 16] = [0.0, 0.01, 0.1, 0.2, 0.3, 0.33333, 0.4, 0.5, 0.6, 0.666666, 0.7, 0.8, 0.9, 0.98, 0.99, 1.0];

pub struct Strategy {
    items: HashMap<(u32, u32), [f32; N_MOVES]>
}

thread_local! {
    pub static STRATEGY: Strategy = Strategy::load();
}

impl Strategy {
    fn load() -> Self {
        let mut last_tag: u32 = 0;
        let mut i = 0;

        let mut all_strategies = HashMap::new();

        loop {
            // try to read tag
            if i >= STRATEGY_DATA.len() { break; }

            let byte1 = STRATEGY_DATA[i];

            let (tag, i2) = 
                if byte1 & 0b11000000 == 0 {
                    // that was the whole tag, and it was a delta
                    (last_tag + STRATEGY_DATA[i] as u32, i + 1)
                } else if byte1 & 0b11000000 == 0b01000000 {
                    (
                        last_tag + u16::from_be_bytes([STRATEGY_DATA[i] & 0b00111111, STRATEGY_DATA[i + 1]]) as u32,
                        i + 2
                    )
                } else {
                    assert!(byte1 & 0b10000000 == 0b10000000);
                    (
                        u32::from_be_bytes([
                            STRATEGY_DATA[i] & 0b01111111,
                            STRATEGY_DATA[i + 1],
                            STRATEGY_DATA[i + 2],
                            STRATEGY_DATA[i + 3],
                        ]), 
                        i + 4
                    )
                };
            i = i2;
            last_tag = tag;

            let n_nonzero = tag & 0b1111;
            let private = (tag >> 4) & 0b11;
            let history = tag >> 6;

            let mut strategy = [0.0; N_MOVES];
            for _ in 0..n_nonzero {
                let ix_val = STRATEGY_DATA[i as usize];
                let ix = ix_val >> 4;
                let val = ix_val & 0b00001111;
                strategy[ix as usize] = VAL_EXPANSION[val as usize];
                assert!((0..N_MOVES).contains(&(ix as usize)));
                i += 1;
            }

            all_strategies.insert((history, private), strategy);
        }

        return Strategy { items: all_strategies }
    }

    pub fn distribution(&self, board: &Board) -> [f32; 9] {
        let player = board.turn % 2;
        let (history, private) = board.infoset.to_key(player==0);
        return self.key_distribution(history, private, &board.possible_moves());
    }

    pub(crate) fn key_distribution(&self, history: u32, private: u32, possible_moves_if_defaulting: &[Move]) -> [f32; 9] {
        let strategy = self.items.get(&(history, private));
         if let Some(s) = strategy {
            let s = *s;
            let sum: f32 = s.iter().sum();
            assert!(sum > 0.0);
            s.map(|x| x/sum)
        } else {
            let mut s2 = [0.0; N_MOVES];
            let n_possible_moves = possible_moves_if_defaulting.len();
            for i in possible_moves_if_defaulting {
                s2[i.0 as usize] = 1.0/(n_possible_moves as f32);
            }
            s2
        }
    }
}

#[test]
fn test_load_strategy() {
    let strategy = Strategy::load();
    println!("{:?}", strategy.key_distribution(0, 0, &[Move(0), Move(1), Move(2)]));
    assert!(1 == 1); // make sure we didn't fuckin crash
}