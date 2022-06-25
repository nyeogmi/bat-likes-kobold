use std::{collections::{HashSet, HashMap, VecDeque, hash_map::Entry}, path::Path, ops::ControlFlow};
use rand::{Rng, distributions::WeightedIndex, prelude::SliceRandom};
use serde::{Serialize, Deserialize};

// == base game ==
const N_MOVES: usize = 9;
const CONTEMPT_ITERATIONS: u64 = 10000; // Iterations for contempt to drop to a very very low number
const DESIRED_ITERATIONS: u64 = 40000; // // NOTE: I've been using 40000 lately, but I drop it to 0 to force a strategy export
const SAVE_EVERY: u64 = 1000;

#[derive(Clone, Copy, PartialEq, Eq)]
enum CellValue { Empty, P0, P1 }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Outcome { Tie, P0Win, P1Win }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Move(usize);

#[derive(Clone, Copy)]
struct Board {
    cells: [CellValue; N_MOVES],
    p0_wants: Outcome,
    p1_wants: Outcome,
    turn: u8,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
struct Infoset { 
    p0_private: u32, 
    p1_private: u32,
    history: u32
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
struct State(u32);

impl Board {
    fn possible_starts() -> Vec<Board> {
        let mut result = vec![];
        for p0 in [Outcome::Tie, Outcome::P0Win, Outcome::P1Win] {
            for p1 in [Outcome::Tie, Outcome::P0Win, Outcome::P1Win] {
                result.push(Board { 
                    cells: [CellValue::Empty; N_MOVES] ,
                    p0_wants: p0,
                    p1_wants: p1,
                    turn: 0,
                });
            }
        };
        result
    }

    fn next_to_move(&self) -> CellValue {
        if self.turn % 2 == 0 { CellValue::P0 } else { CellValue::P1 }
    }

    fn possible_moves(&self) -> Vec<Move> {
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

    fn play(&mut self, m: Move) {
        assert!(m.0 < self.cells.len() && self.cells[m.0] == CellValue::Empty);
        self.cells[m.0] = self.next_to_move();
        self.turn += 1
    }

    fn score(&self) -> Option<(Outcome, i8, i8)> {
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

// == mapper ==
impl Board {
    pub fn to_base_infoset(&self) -> Infoset {
        Infoset {
            p0_private: self.p0_wants.to_smallint(),
            p1_private: self.p1_wants.to_smallint(),
            history: 1
        }
    }

    pub fn to_state(&self) -> State {
        let mut value = 0;

        value *= 3;
        value += self.p0_wants.to_smallint();
        value *= 3;
        value += self.p1_wants.to_smallint();

        for i in self.cells {
            value *= 3;
            value += i.to_smallint();
        }

        State(value)
    }
}

impl CellValue {
    fn to_smallint(self) -> u32 {
        match self {
            CellValue::Empty => 0,
            CellValue::P0 => 1,
            CellValue::P1 => 2,
        }
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


fn map_game() -> CFR {
    let mut reached: HashSet<State> = HashSet::new();
    let mut state_initial: HashMap<State, Infoset> = HashMap::new();
    let mut state_edges: HashMap<State, [Option<State>; N_MOVES]> = HashMap::new();
    let mut state_score: HashMap<State, (i8, i8)> = HashMap::new();

    let mut queue: VecDeque<(Option<(State, Move)>, Board)> = VecDeque::new();
    for start in Board::possible_starts() {
        queue.push_back((None, start))
    }

    while let Some((predecessor, board)) = queue.pop_front() {
        let state = board.to_state();

        match predecessor {
            None => { state_initial.insert(state, board.to_base_infoset()); }
            Some((pred, m)) => {
                state_edges.entry(pred).or_insert_with(|| [None; N_MOVES])[m.0] = Some(state);
            }
        }

        if reached.contains(&state) { continue; }
        reached.insert(state);

        if let Some(s) = board.score() {
            state_score.insert(state, (s.1, s.2));
        }

        for m in board.possible_moves() {
            let mut b2 = board;
            b2.play(m);
            queue.push_back((Some((state, m)), b2));
        }
    }

    let mut initial: Vec<(State, Infoset)> = state_initial.iter().map(|(x, y)| (*x, *y)).collect();
    initial.sort_unstable_by_key(|(s, _)| s.0);

    let max_state = reached.iter().max_by_key(|s| s.0).expect("expected at least one state");

    let mut states: Vec<Option<StateNode>> = vec![None; max_state.0 as usize + 1];

    for s in reached {
        let successors = state_edges.get(&s).map(|x| *x).unwrap_or_default();
        states[s.0 as usize] = Some(StateNode {
            successors,
            score: state_score.get(&s).cloned(),
        });
    }

    CFR { trained_iterations: 0, initial, states, infosets: HashMap::new() }
}

// CFR
#[derive(Clone, Debug, Serialize, Deserialize)]  // no Copy: you probably don't want to copy these as they are frequently mutated in place
struct InfosetNode {
    legal: [bool; N_MOVES],
    regret_sum: [f32; N_MOVES],
    strategy_sum: [f32; N_MOVES],
}

impl InfosetNode {
    fn get_strategy(&mut self, realization_weight: f32, contempt: f32) -> [f32; N_MOVES] {
        let mut strategy = [0.0; N_MOVES];
        for i in 0..N_MOVES { strategy[i] = self.regret_sum[i].max(0.0) }
        self._normalize(&mut strategy);
        for i in 0..N_MOVES {
            self.strategy_sum[i] += strategy[i] * realization_weight
        }

        if contempt > 0.0 {
            let n_legal_moves = self.legal.iter().filter(|i| **i).count();
            for i in 0..N_MOVES {
                if self.legal[i] { strategy[i] = strategy[i] * (1.0 - contempt) + contempt / n_legal_moves as f32 }
            }
            self._normalize(&mut strategy);
        }
        strategy
    }

    fn get_average_strategy(&self) -> [f32; N_MOVES] {
        let mut strat = self.strategy_sum;
        self._normalize(&mut strat);
        strat
    }

    fn _normalize(&self, strategy: &mut [f32; N_MOVES]) {
        let total_points: f32 = strategy.iter().sum();
        if total_points == 0.0 {
            let mut n_possible_actions = 0;
            for l  in self.legal { if l { n_possible_actions += 1; } }

            for i in 0..N_MOVES {
                strategy[i] = 0.0;
                if self.legal[i] {
                    strategy[i] = 1.0/n_possible_actions as f32;
                }
            }
        } else {
            for i in 0..N_MOVES {
                strategy[i] /= total_points;
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
struct StateNode {
    successors: [Option<State>; N_MOVES],
    score: Option<(i8, i8)>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CFR {
    trained_iterations: u64,
    initial: Vec<(State, Infoset)>,
    states: Vec<Option<StateNode>>,
    infosets: HashMap<(u32, u32), InfosetNode>,
}

impl CFR {
    fn with_infoset_node<T>(&mut self, infoset: Infoset, for_p0: bool, state: &StateNode, f: impl FnOnce(&mut InfosetNode) -> T) -> T {
        let mut legal = [false; N_MOVES];
        for i in 0..N_MOVES {
            if let Some(_) = state.successors[i] { legal[i] = true; }
        }

        let key = (infoset.history, if for_p0 { infoset.p0_private } else { infoset.p1_private });
        match self.infosets.entry(key) {
            Entry::Occupied(mut o) => {
                let iset =  o.get_mut();
                assert!(iset.legal == legal);
                f(iset)
            }
            Entry::Vacant(v) =>  {
                f(v.insert(InfosetNode { legal, regret_sum: [0.0; N_MOVES], strategy_sum: [0.0; N_MOVES]}))
            }
        }
    }

    fn train(&mut self, contempt: f32) -> f32 {
        let initial = self.initial.clone();
        let mut util = 0.0;
        for (init_state, init_infoset) in initial.iter() {
            util += self._train(*init_state, *init_infoset, contempt, 0, 1.0, 1.0);
        }
        util /= initial.len() as f32;
        self.trained_iterations += 1;
        return util;
    }

    fn _train(&mut self, state: State, infoset: Infoset, contempt: f32, turn: usize, p0: f32, p1: f32) -> f32 {
        let player = turn % 2;
        let node = self.states[state.0 as usize].expect("missing state");

        if let Some((sc_p0, sc_p1)) = node.score {
            let mut sc_p0_adjusted = sc_p0 as f32;
            let mut sc_p1_adjusted = sc_p1 as f32;

            // strongly prefer to win in fewer turns
            // this probably isn't good for its overall play, but _is_ more humanlike
            let adj_turn = (turn / 2) as f32;
            let adj_turn_multiplier = (4.0 - adj_turn).max(0.0)/4.0;
            if sc_p0_adjusted > 0.0 { sc_p0_adjusted += contempt * adj_turn_multiplier }
            if sc_p1_adjusted > 0.0 { sc_p1_adjusted += contempt * adj_turn_multiplier }

            let mut utility = (sc_p0_adjusted - sc_p1_adjusted) as f32;
            if player == 1 { utility = -utility }
            return utility
        }

        let strategy = 
            self.with_infoset_node(infoset, player == 0, &node, |n| 
                n.get_strategy(if player == 0 { p0 } else { p1 }, contempt)
            );

        let mut util = [0.0; N_MOVES];
        let mut node_util = 0.0;

        for m in 0..N_MOVES {
            if let Some(successor) = node.successors[m] {
                util[m] = if player == 0 {
                    -self._train(successor, infoset.cons(Move(m)), contempt, turn + 1, p0 * strategy[m], p1)
                } else {
                    -self._train(successor, infoset.cons(Move(m)), contempt, turn + 1, p0, p1 * strategy[m])
                };
                node_util += strategy[m] * util[m];
            }
        }

        for m in 0..N_MOVES {
            if let Some(_) = node.successors[m] {  // if the move was legal
                let regret = util[m] - node_util;
                self.with_infoset_node(infoset, player == 0, &node, |n| {
                    n.regret_sum[m] += if player == 0 { p0 } else { p1 } * regret;
                })
            }
        }

        node_util
    }
}


fn main() {
    let path = Path::new("cfr.dat");
    println!("loading CFR data");
    let mut cfr = match std::fs::read(&path) {
        Ok(data) => match bincode::deserialize::<CFR>(&data) {
            Ok(cfr) => cfr,
            Err(err) => {
                println!("... couldn't load CFR data, but found file; noping out ({})", err);
                return
            }
        }
        Err(err) => {
            match path.try_exists() {
                Ok(true) => { 
                    println!("... couldn't load CFR data, but file definitely exists. noping out ({})", err); 
                    return 
                }
                Ok(false) => {
                    println!("CFR data doesn't exist.");
                    println!("mapping game...");
                    let cfr = map_game();

                    if let ControlFlow::Break(_) = save_cfr(&cfr, path) { return; }
                    cfr
                }
                Err(err) => {
                    println!("... couldn't figure out if CFR data exists. noping out ({})", err);
                    return
                }
            }
        }
    };

    while cfr.trained_iterations < DESIRED_ITERATIONS {
        println!("training: iteration {}", cfr.trained_iterations);
        let contempt = (0.5 * (1.0 - cfr.trained_iterations as f32 / CONTEMPT_ITERATIONS as f32)).max(0.01);
        let util = cfr.train(contempt);
        println!("average utility: {}", util);

        if cfr.trained_iterations % SAVE_EVERY == 0 {
            if let ControlFlow::Break(_) = save_cfr(&cfr, path) { return; }
        }
    }
    if let ControlFlow::Break(_) = save_cfr(&cfr, path) { return; }

    // let mut rng = rand::prelude::StdRng::seed_from_u64(4);
    let mut rng = rand::thread_rng();
    play_game(&mut rng, &mut cfr);

    let strategydata = export_strategy(&mut cfr);
    match std::fs::write("strategy.dat", strategydata) {
        Ok(_) => println!("... exported strategy!"),
        Err(e) => { println!("... could not export strategy! {}", e)}
    }
}

fn export_strategy(cfr: &mut CFR) -> Vec<u8> {
    fn is_interesting(node: &InfosetNode, strategy: &[f32; N_MOVES]) -> bool {
        // return whether the distribution is substantially different from picking uniformly at random
        let mut default_strategy = [0.0; N_MOVES];
        node._normalize(&mut default_strategy);

        // calculate bhattacharyya distance
        let mut bhat = 0.0;
        for i in 0..N_MOVES {
            bhat += (default_strategy[i] * strategy[i]).sqrt();
        }
        return bhat < 0.9
    }

    fn simplify(strategy: [f32; N_MOVES]) -> [u8; N_MOVES] {
        const POSSIBILITIES: [f32; 16] = [0.0, 0.01, 0.1, 0.2, 0.3, 0.33333, 0.4, 0.5, 0.6, 0.666666, 0.7, 0.8, 0.9, 0.98, 0.99, 1.0];

        fn simplify_term(term: f32) -> u8 {
            let dist = POSSIBILITIES.map(|p| {
                ((term - p).abs() * 100000.0) as u32
            });

            return (0..POSSIBILITIES.len()).min_by_key(|i| dist[*i]).unwrap() as u8
        }

        strategy.map(simplify_term)
    }

    let mut out: Vec<u8> = Vec::new();
    let mut sorted_infosets: Vec<_> = cfr.infosets.iter().collect();
    sorted_infosets.sort_by_key(|((history, private), _)| (history, private));

    let mut last_tag: u32 = 0;

    for ((history, private), node) in sorted_infosets.iter() {
        let strategy = node.get_average_strategy();
        if !is_interesting(node, &strategy) { continue; }

        let simp: [u8; N_MOVES] = simplify(strategy);  // note: must be 9 long
        let n_nonzero = simp.iter().filter(|x| **x != 0).count() as u32;

        assert!(n_nonzero > 0);
        assert!(n_nonzero <= 7);

        assert!(history & 0x03ffffff == *history);
        assert!(private & 0b11 == *private);

        let tag = history << 6 | private << 4 | n_nonzero;

        let diff_tag = tag - last_tag;

        if diff_tag < 64 {
            let bytes = (diff_tag as u8).to_be_bytes();
            assert!(bytes[0] & 0b00111111 == bytes[0]);
            out.extend(bytes);
        } else if diff_tag < 64 * 256 {
            let mut bytes = (diff_tag as u16).to_be_bytes();
            assert!(bytes[0] & 0b00111111 == bytes[0]);
            bytes[0] |= 0b01000000;
            // TODO: bitwise tag
            out.extend(bytes);
        } else if tag < 128 * 256 * 256 * 256 {
            let mut bytes = (tag as u32).to_be_bytes();
            assert!(bytes[0] & 0b01111111 == bytes[0]);
            bytes[0] |= 0b10000000;
            out.extend(bytes);
        } else {
            panic!("tag is too big: {}", tag);
        }
        last_tag = tag;

        for (i, value) in simp.iter().enumerate() {
            if *value != 0 { out.push((i as u8) << 4 | value) }
        }
    }

    return out;
}

fn save_cfr(cfr: &CFR, path: &Path) -> ControlFlow<()> {
    match bincode::serialize::<CFR>(cfr) {
        Ok(o) => {
            match std::fs::write(path, o) {
                Ok(()) => { println!("... saved!"); }
                Err(err) => {
                    println!("couldn't save CFR data. noping out ({})", err);
                    return ControlFlow::Break(())
                }
            }
        }
        Err(err) => {
            println!("... couldn't serialize CFR data. noping out ({})", err);
            return ControlFlow::Break(())
        }
    }
    ControlFlow::Continue(())
}

fn play_game(rng: &mut impl Rng, cfr: &mut CFR) {
    let possible_starts = Board::possible_starts();

    let mut board = possible_starts.choose(rng).unwrap().clone();
    let mut infoset = board.to_base_infoset();

    loop {
        if let Some((outcome, p0, p1)) = board.score() {
            draw_board(&board);
            println!("result: {:?} ({}/{})", outcome, p0, p1);
            println!("p0 wanted: {:?}", board.p0_wants);
            println!("p1 wanted: {:?}", board.p1_wants);
            return
        }

        let next_move = if board.turn % 2 == 0 {
            let possible_moves = board.possible_moves();
            draw_board(&board);
            println!("What's your move, human? ({:?})", possible_moves);
            /*
            possible_moves[0]
            */
            let strategy = cfr.with_infoset_node(
                infoset, 
                true, 
                &cfr.states[board.to_state().0 as usize].unwrap(), 
                |i| i.get_average_strategy()
            );
            let mv = sample_strategy(rng, strategy, &possible_moves);
            mv
        }
        else {
            let possible_moves = board.possible_moves();
            draw_board(&board);
            println!("What's your move, robot? ({:?})", possible_moves);

            let strategy = cfr.with_infoset_node(
                infoset, 
                false, 
                &cfr.states[board.to_state().0 as usize].unwrap(), 
                |i| i.get_average_strategy()
            );
            let mv = sample_strategy(rng, strategy, &possible_moves);
            mv
        };

        board.play(next_move);
        infoset = infoset.cons(next_move)
    }

}

fn draw_board(board: &Board) {
    let nice_cell = |x| match x {
        CellValue::Empty => '-',
        CellValue::P0 => 'D',
        CellValue::P1 => 'Z',
    };
    // player move
    println!("{} {} {}\n{} {} {}\n{} {} {}", 
        nice_cell(board.cells[0]), nice_cell(board.cells[1]), nice_cell(board.cells[2]),
        nice_cell(board.cells[3]), nice_cell(board.cells[4]), nice_cell(board.cells[5]),
        nice_cell(board.cells[6]), nice_cell(board.cells[7]), nice_cell(board.cells[8]),
    );
}

fn sample_strategy(rng: &mut impl Rng, strategy: [f32; N_MOVES], possible_moves: &[Move]) -> Move {
    if possible_moves.len() == 0 { panic!("should never happen"); }

    // NOTE: This allocates
    let weights = WeightedIndex::new(strategy).unwrap();

    loop {
        let ix = rng.sample(&weights);
        println!("{:?} {}", strategy, ix);
        if possible_moves.contains(&Move(ix)) {
            return Move(ix);
        } 

        // just in case there's somehow some nonzero probability, we loop
    }
}

impl Infoset {
    fn cons(&self, m: Move) -> Infoset {
        let mut is2 = *self;
        is2.history *= N_MOVES as u32;
        is2.history += m.0 as u32;
        is2
    }
}
