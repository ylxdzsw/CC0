#![feature(generic_const_exprs)]

use std::{collections::BTreeMap, io::{Write, Read}, sync::{atomic::{AtomicBool, AtomicU64, Ordering}, RwLock}};

const USAGE: &str = "
Usage: super_mcts playout [count=10000] [threads=auto]
       super_mcts analyze
";

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let args: Vec<_> = args.iter().map(|x| &x[..]).collect();

    let db = DB::<SmallBoard>::new();
    if std::path::Path::new("result.bin").exists() {
        db.load()
    } else {
        db.query(&Game::new().pieces, || 0.0); // ensure root node
    }

    match args[..] {
        ["playout"] => playout(10000, &db, 0),
        ["playout", count] => playout(count.parse().unwrap(), &db, 0),
        ["playout", count, threads] => playout(count.parse().unwrap(), &db, threads.parse().unwrap()),
        ["analyze"] => analyze(&db),
        _ => eprint!("{}", USAGE)
    }
}

struct AtomicF64(AtomicU64);

impl AtomicF64 {
    fn new(value: f64) -> Self {
        Self(AtomicU64::new(value.to_bits()))
    }
    fn store(&self, value: f64, ordering: Ordering) {
        self.0.store(value.to_bits(), ordering)
    }
    fn load(&self, ordering: Ordering) -> f64 {
        f64::from_bits(self.0.load(ordering))
    }
}

struct DB<Board: BoardDef> where [(); 2 * Board::N_PIECES]: {
    tree: RwLock<BTreeMap<[u8; 2 * Board::N_PIECES], (AtomicU64, AtomicF64)>>
}

impl<Board: BoardDef> DB<Board> where [(); 2 * Board::N_PIECES]: {
    fn new() -> Self {
        Self { tree: RwLock::new(BTreeMap::new()) }
    }

    fn query(&self, pieces: &[u8; 2 * Board::N_PIECES], default: impl Fn() -> f64) -> (u64, f64) {
        // try to not write lock the tree
        {
            let tree = self.tree.read().unwrap();
            if let Some((n, v)) = tree.get(pieces) {
                return (n.load(Ordering::Relaxed), v.load(Ordering::Relaxed))
            }
        }

        // initialize
        {
            let mut tree = self.tree.write().unwrap();
            let default_value = default();
            tree.insert(*pieces, (AtomicU64::new(0), AtomicF64::new(default_value)));
            return (0, default_value)
        }
    }

    fn update(&self, pieces: &[u8; 2 * Board::N_PIECES], new_value: f64, learning_rate: f64) {
        let tree = self.tree.read().unwrap();
        if let Some((n, v)) = tree.get(pieces) {
            n.fetch_add(1, Ordering::Relaxed);
            v.store(v.load(Ordering::Relaxed) * (1.0 - learning_rate) + new_value * learning_rate, Ordering::Relaxed);
        } else {
            unreachable!()
        }
    }

    fn record_ending(&self, pieces: &[u8; 2 * Board::N_PIECES]) {
        let tree = self.tree.read().unwrap();
        if let Some((n, _)) = tree.get(pieces) {
            n.fetch_add(1, Ordering::Relaxed);
        } else {
            unreachable!()
        }
    }

    fn dump(&self) {
        let tree = self.tree.read().unwrap();
        let result_file = std::fs::File::create("result.bin").unwrap();
        let mut result_file_writer = std::io::BufWriter::new(result_file);
        for (pieces, (n, v)) in tree.iter() {
            let (n, v) = (n.load(Ordering::Relaxed), v.load(Ordering::Relaxed));
            if n >= 5 {
                result_file_writer.write_all(pieces).unwrap();
                result_file_writer.write_all(&n.to_le_bytes()).unwrap();
                result_file_writer.write_all(&v.to_le_bytes()).unwrap();
            }
        }
        result_file_writer.flush().unwrap();
    }

    fn load(&self) {
        let mut tree = self.tree.write().unwrap();
        let result_file = std::fs::File::open("result.bin").unwrap();
        let mut result_file_reader = std::io::BufReader::new(result_file);
        loop {
            let mut pieces = [0; 2 * Board::N_PIECES];
            let mut n = [0; 8];
            let mut v = [0; 8];
            if result_file_reader.read_exact(&mut pieces).is_err() {
                break;
            }
            result_file_reader.read_exact(&mut n).unwrap();
            result_file_reader.read_exact(&mut v).unwrap();
            let (n, v) = (u64::from_le_bytes(n), f64::from_le_bytes(v));
            tree.insert(pieces, (AtomicU64::new(n), AtomicF64::new(v)));
        }
    }
}

static STOP: AtomicBool = AtomicBool::new(false);

fn playout<Board: BoardDef>(count: usize, db: &DB<Board>, threads: usize) where [(); 2 * Board::N_PIECES]:, [(); Board::BOARD_SIZE]: {
    let thread_count = if threads != 0 {
        threads
    } else {
        std::thread::available_parallelism().map(|x| x.get()).unwrap_or(8)
    };
    let game_count_each_thread = count / thread_count;

    ctrlc::set_handler(move || {
        STOP.store(true, Ordering::Relaxed);
    }).unwrap();

    std::thread::scope(|s| {
        for _ in 0..thread_count {
            s.spawn(|| {
                for _ in 0..game_count_each_thread {
                    if STOP.load(Ordering::Relaxed) {
                        break;
                    }
                    playout_game(Game::new(), &db);
                }
            });
        }
    });

    db.dump()
}

fn playout_game<Board: BoardDef>(game: Game<Board>, db: &DB<Board>) -> f64 where [(); 2 * Board::N_PIECES]:, [(); Board::BOARD_SIZE]: {
    let (mut next_states, _) = game.expand(false);

    if next_states.is_empty() {
        db.record_ending(&game.pieces);
        return game.heuristic()
    }

    random_shuffle(&mut next_states);

    let (_visits, mut values) = next_states.iter()
        .map(|g| db.query(&g.pieces, || g.heuristic()))
        .unzip::<_, _, Vec<_>, Vec<_>>();

    let mut probs = values.clone();
    if game.is_p2_moving_next() {
        probs.iter_mut().for_each(|x| *x = -*x);
    }
    softmax(&mut probs, 1.0);

    let i = sample_categorical(probs.iter().cloned());

    let next_state = next_states[i].clone();

    values[i] = playout_game(next_state, db);

    let updated_value = values.into_iter().zip(probs.into_iter()).map(|(value, prob)| value * prob).sum::<f64>();

    db.update(&game.pieces, updated_value, 0.2 * game.turn as f64 / Board::TURN_LIMIT as f64);

    updated_value
}

fn analyze<Board: BoardDef>(db: &DB<Board>) where [(); 2 * Board::N_PIECES]:, [(); Board::BOARD_SIZE]: {
    let (root_visit, root_value) = db.query(&Game::new().pieces, || 0.0);
    println!("root_visit: {}, root_value: {}", root_visit, root_value);

    let mut total_turns = 0;
    for _ in 0..32 {
        total_turns += random_play(&Game::new(), &db).len();
    }
    println!("average turns: {}", total_turns as f32 / 32.);

    println!("example play:");
    for record in random_play(&Game::new(), &db) {
        println!("=== turn {} ===", record.game.turn);
        println!("scores: {}, {}", record.game.p1_score(), record.game.p2_score());
        println!("values: {:?}", record.children_values);
        println!("probs: {:?}", record.probs);
        println!("choice: {}", record.choice);
        println!("visit: {}", record.visit);
        println!("action: {} -> {}", record.from, record.to);
    }
}

struct Record<Board: BoardDef> where [(); 2 * Board::N_PIECES]:, [(); Board::BOARD_SIZE]: {
    game: Game<Board>,
    from: Position,
    to: Position,
    children_values: Vec<f64>,
    probs: Vec<f64>,
    choice: usize,
    visit: u64,
}

fn random_play<Board: BoardDef>(game: &Game<Board>, db: &DB<Board>) -> Vec<Record<Board>> where [(); 2 * Board::N_PIECES]:, [(); Board::BOARD_SIZE]: {
    let (next_states, actions) = game.expand(true);
    if next_states.is_empty() {
        return vec![]
    }

    let (visits, _values) = next_states.iter()
        .map(|g| db.query(&g.pieces, || g.heuristic()))
        .unzip::<_, _, Vec<_>, Vec<_>>();

    for (mut visit, state) in visits.into_iter().zip(next_states.iter()) {
        while visit < 5 {
            playout_game(state.clone(), db);
            visit += 1;
        }
    }

    let (visits, values) = next_states.iter()
        .map(|g| db.query(&g.pieces, || g.heuristic()))
        .unzip::<_, _, Vec<_>, Vec<_>>();

    let mut probs: Vec<_> = values.iter().map(|x| *x as f64).collect();
    if game.is_p2_moving_next() {
        probs.iter_mut().for_each(|x| *x = -*x);
    }
    softmax(&mut probs, 0.5);

    let i = sample_categorical(probs.iter().cloned());
    let mut result = vec![Record {
        game: game.clone(),
        from: actions[i].0,
        to: actions[i].1,
        children_values: values,
        probs,
        choice: i,
        visit: visits[i],
    }];
    for x in random_play(&next_states[i], db) {
        result.push(x);
    }
    return result
}

type Position = u8;

const INVALID_POSITION: Position = Position::MAX;

trait BoardDef {
    const N_PIECES: usize;
    const BOARD_SIZE: usize;
    const TURN_LIMIT: usize = 6 * Self::N_PIECES;
    const ADJ_MATRIX: &'static [[Position; 6]];
    const P1_BASE: &'static [Position];
    const P2_BASE: &'static [Position];
    const STARTING_STATE: &'static [Position];
    const P1_SCORE: &'static [u64];
    const P2_SCORE: &'static [u64];
    const MIN_SCORE: u64;
}

struct SmallBoard;

impl BoardDef for SmallBoard {
    const N_PIECES: usize = 6;
    const BOARD_SIZE: usize = 73;
    const ADJ_MATRIX: &'static [[Position; 6]] = &[[1,2,255,255,255,255],[3,4,2,0,255,255],[4,5,255,255,0,1],[9,10,4,1,255,255],[10,11,5,2,1,3],[11,12,255,255,2,4],[255,16,7,255,255,255],[16,17,8,255,255,6],[17,18,9,255,255,7],[18,19,10,3,255,8],[19,20,11,4,3,9],[20,21,12,5,4,10],[21,22,13,255,5,11],[22,23,14,255,255,12],[23,24,15,255,255,13],[24,255,255,255,255,14],[255,25,17,7,6,255],[25,26,18,8,7,16],[26,27,19,9,8,17],[27,28,20,10,9,18],[28,29,21,11,10,19],[29,30,22,12,11,20],[30,31,23,13,12,21],[31,32,24,14,13,22],[32,255,255,15,14,23],[255,33,26,17,16,255],[33,34,27,18,17,25],[34,35,28,19,18,26],[35,36,29,20,19,27],[36,37,30,21,20,28],[37,38,31,22,21,29],[38,39,32,23,22,30],[39,255,255,24,23,31],[40,41,34,26,25,255],[41,42,35,27,26,33],[42,43,36,28,27,34],[43,44,37,29,28,35],[44,45,38,30,29,36],[45,46,39,31,30,37],[46,47,255,32,31,38],[48,49,41,33,255,255],[49,50,42,34,33,40],[50,51,43,35,34,41],[51,52,44,36,35,42],[52,53,45,37,36,43],[53,54,46,38,37,44],[54,55,47,39,38,45],[55,56,255,255,39,46],[57,58,49,40,255,255],[58,59,50,41,40,48],[59,60,51,42,41,49],[60,61,52,43,42,50],[61,62,53,44,43,51],[62,63,54,45,44,52],[63,64,55,46,45,53],[64,65,56,47,46,54],[65,66,255,255,47,55],[255,255,58,48,255,255],[255,255,59,49,48,57],[255,255,60,50,49,58],[255,67,61,51,50,59],[67,68,62,52,51,60],[68,69,63,53,52,61],[69,255,64,54,53,62],[255,255,65,55,54,63],[255,255,66,56,55,64],[255,255,255,255,56,65],[255,70,68,61,60,255],[70,71,69,62,61,67],[71,255,255,63,62,68],[255,72,71,68,67,255],[72,255,255,69,68,70],[255,255,255,71,70,255]];
    const P1_BASE: &'static [Position] = &[0, 1, 2, 3, 4, 5];
    const P2_BASE: &'static [Position] = &[67, 68, 69, 70, 71, 72];
    const STARTING_STATE: &'static [Position] = &[0, 1, 2, 3, 4, 5, 67, 68, 69, 70, 71, 72];
    const P1_SCORE: &'static [u64] = &[12, 11, 11, 10, 10, 10, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 8, 8, 8, 8, 8, 8, 8, 8, 8, 7, 7, 7, 7, 7, 7, 7, 7, 6, 6, 6, 6, 6, 6, 6, 6, 5, 5, 5, 5, 5, 5, 6, 6, 5, 4, 4, 4, 4, 4, 5, 6, 6, 5, 4, 3, 3, 3, 3, 4, 5, 6, 2, 2, 2, 1, 1, 0];
    const P2_SCORE: &'static [u64] = &[0, 1, 1, 2, 2, 2, 6, 5, 4, 3, 3, 3, 3, 4, 5, 6, 6, 5, 4, 4, 4, 4, 4, 5, 6, 6, 5, 5, 5, 5, 5, 5, 6, 6, 6, 6, 6, 6, 6, 6, 7, 7, 7, 7, 7, 7, 7, 7, 8, 8, 8, 8, 8, 8, 8, 8, 8, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 10, 10, 10, 11, 11, 12];
    const MIN_SCORE: u64 = 8;
}

// const BOARD_N_PIECES: usize = 10;
// const BOARD_SIZE: usize = 121;
// const BOARD_TURN_LIMIT: usize = 6 * BOARD_N_PIECES;
// const BOARD_ADJ_MATRIX: &'static [[Position; 6]; BOARD_SIZE] = &[[1,2,255,255,255,255],[3,4,2,0,255,255],[4,5,255,255,0,1],[6,7,4,1,255,255],[7,8,5,2,1,3],[8,9,255,255,2,4],[14,15,7,3,255,255],[15,16,8,4,3,6],[16,17,9,5,4,7],[17,18,255,255,5,8],[255,23,11,255,255,255],[23,24,12,255,255,10],[24,25,13,255,255,11],[25,26,14,255,255,12],[26,27,15,6,255,13],[27,28,16,7,6,14],[28,29,17,8,7,15],[29,30,18,9,8,16],[30,31,19,255,9,17],[31,32,20,255,255,18],[32,33,21,255,255,19],[33,34,22,255,255,20],[34,255,255,255,255,21],[255,35,24,11,10,255],[35,36,25,12,11,23],[36,37,26,13,12,24],[37,38,27,14,13,25],[38,39,28,15,14,26],[39,40,29,16,15,27],[40,41,30,17,16,28],[41,42,31,18,17,29],[42,43,32,19,18,30],[43,44,33,20,19,31],[44,45,34,21,20,32],[45,255,255,22,21,33],[255,46,36,24,23,255],[46,47,37,25,24,35],[47,48,38,26,25,36],[48,49,39,27,26,37],[49,50,40,28,27,38],[50,51,41,29,28,39],[51,52,42,30,29,40],[52,53,43,31,30,41],[53,54,44,32,31,42],[54,55,45,33,32,43],[55,255,255,34,33,44],[255,56,47,36,35,255],[56,57,48,37,36,46],[57,58,49,38,37,47],[58,59,50,39,38,48],[59,60,51,40,39,49],[60,61,52,41,40,50],[61,62,53,42,41,51],[62,63,54,43,42,52],[63,64,55,44,43,53],[64,255,255,45,44,54],[65,66,57,47,46,255],[66,67,58,48,47,56],[67,68,59,49,48,57],[68,69,60,50,49,58],[69,70,61,51,50,59],[70,71,62,52,51,60],[71,72,63,53,52,61],[72,73,64,54,53,62],[73,74,255,55,54,63],[75,76,66,56,255,255],[76,77,67,57,56,65],[77,78,68,58,57,66],[78,79,69,59,58,67],[79,80,70,60,59,68],[80,81,71,61,60,69],[81,82,72,62,61,70],[82,83,73,63,62,71],[83,84,74,64,63,72],[84,85,255,255,64,73],[86,87,76,65,255,255],[87,88,77,66,65,75],[88,89,78,67,66,76],[89,90,79,68,67,77],[90,91,80,69,68,78],[91,92,81,70,69,79],[92,93,82,71,70,80],[93,94,83,72,71,81],[94,95,84,73,72,82],[95,96,85,74,73,83],[96,97,255,255,74,84],[98,99,87,75,255,255],[99,100,88,76,75,86],[100,101,89,77,76,87],[101,102,90,78,77,88],[102,103,91,79,78,89],[103,104,92,80,79,90],[104,105,93,81,80,91],[105,106,94,82,81,92],[106,107,95,83,82,93],[107,108,96,84,83,94],[108,109,97,85,84,95],[109,110,255,255,85,96],[255,255,99,86,255,255],[255,255,100,87,86,98],[255,255,101,88,87,99],[255,255,102,89,88,100],[255,111,103,90,89,101],[111,112,104,91,90,102],[112,113,105,92,91,103],[113,114,106,93,92,104],[114,255,107,94,93,105],[255,255,108,95,94,106],[255,255,109,96,95,107],[255,255,110,97,96,108],[255,255,255,255,97,109],[255,115,112,103,102,255],[115,116,113,104,103,111],[116,117,114,105,104,112],[117,255,255,106,105,113],[255,118,116,112,111,255],[118,119,117,113,112,115],[119,255,255,114,113,116],[255,120,119,116,115,255],[120,255,255,117,116,118],[255,255,255,119,118,255]];
// const BOARD_P1_BASE: &'static [Position; BOARD_N_PIECES] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
// const BOARD_P2_BASE: &'static [Position; BOARD_N_PIECES] = &[111, 112, 113, 114, 115, 116, 117, 118, 119, 120];
// const STARTING_STATE: &'static [Position; 1 + 2 * BOARD_N_PIECES] = &[0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 111, 112, 113, 114, 115, 116, 117, 118, 119, 120];
// const BOARD_P1_SCORE: &'static [usize; BOARD_SIZE] = &[16, 15, 15, 14, 14, 14, 13, 13, 13, 13, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 7, 7, 7, 7, 7, 7, 7, 7, 8, 8, 7, 6, 6, 6, 6, 6, 6, 6, 7, 8, 8, 7, 6, 5, 5, 5, 5, 5, 5, 6, 7, 8, 8, 7, 6, 5, 4, 4, 4, 4, 4, 5, 6, 7, 8, 3, 3, 3, 3, 2, 2, 2, 1, 1, 0];
// const BOARD_P2_SCORE: &'static [usize; BOARD_SIZE] = &[0, 1, 1, 2, 2, 2, 3, 3, 3, 3, 8, 7, 6, 5, 4, 4, 4, 4, 4, 5, 6, 7, 8, 8, 7, 6, 5, 5, 5, 5, 5, 5, 6, 7, 8, 8, 7, 6, 6, 6, 6, 6, 6, 6, 7, 8, 8, 7, 7, 7, 7, 7, 7, 7, 7, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 13, 13, 13, 13, 14, 14, 14, 15, 15, 16];
// const MIN_SCORE: usize = 20;


struct Game<Board: BoardDef> where [(); 2 * Board::N_PIECES]: {
    turn: usize, // Initial state (no players has moved and player 1 is about to move next) has turn = 0.
    pieces: [Position; 2 * Board::N_PIECES],
}

impl<Board: BoardDef> Clone for Game<Board> where [(); 2 * Board::N_PIECES]: {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Board: BoardDef> Copy for Game<Board> where [(); 2 * Board::N_PIECES]: {}

impl<Board: BoardDef> Game<Board> where [(); 2 * Board::N_PIECES]: {
    fn new() -> Self {
        Game::new_with_pieces(Board::STARTING_STATE.try_into().unwrap())
    }

    fn is_p1_moving_next(&self) -> bool {
        self.turn % 2 == 0
    }

    fn is_p2_moving_next(&self) -> bool {
        self.turn % 2 == 1
    }

    fn p1_pieces_slice(&self) -> &[u8] {
        &self.pieces[..Board::N_PIECES]
    }

    fn p1_pieces_slice_mut(&mut self) -> &mut [u8] {
        &mut self.pieces[..Board::N_PIECES]
    }

    fn p2_pieces_slice(&self) -> &[u8] {
        &self.pieces[Board::N_PIECES..]
    }

    fn p2_pieces_slice_mut(&mut self) -> &mut [u8] {
        &mut self.pieces[Board::N_PIECES..]
    }

    fn has_piece(&self, piece: Position) -> bool {
        self.pieces.binary_search(&piece).is_ok()
    }

    fn move_to(&self, from: Position, to: Position) -> Self {
        let mut result = self.clone();
        result.turn += 1;

        let moving_slice = if self.is_p1_moving_next() {
            result.p1_pieces_slice_mut()
        } else {
            result.p2_pieces_slice_mut()
        };

        let from_idx = moving_slice.binary_search(&from).unwrap();
        moving_slice[from_idx] = to;
        moving_slice.sort_unstable();

        result
    }

    fn p1_score(&self) -> u64 {
        self.p1_pieces_slice().iter().map(|&p| Board::P1_SCORE[p as usize]).sum()
    }

    fn p2_score(&self) -> u64 {
        self.p2_pieces_slice().iter().map(|&p| Board::P2_SCORE[p as usize]).sum()
    }

    fn expand(&self, record_actions: bool) -> (Vec<Game<Board>>, Vec<(Position, Position, [Position; Board::BOARD_SIZE])>) {
        // early finish
        if self.p1_score() == Board::MIN_SCORE || self.p2_score() == Board::MIN_SCORE {
            return (vec![], vec![])
        }

        // force finish
        if self.turn > Board::TURN_LIMIT {
            return (vec![], vec![])
        }

        let mut next_states = vec![];
        let mut actions = vec![];

        let moving_slice = if self.is_p1_moving_next() {
            self.p1_pieces_slice()
        } else {
            self.p2_pieces_slice()
        };

        for &piece in moving_slice {
            let paths = self.possible_moves_with_path(piece);

            for dest in paths.into_iter().enumerate().filter(|&(dest, from)| from != INVALID_POSITION && dest as u8 != piece).map(|(dest, _)| dest as Position) {
                next_states.push(self.move_to(piece, dest));
                if record_actions {
                    actions.push((piece, dest, paths));
                }
            }
        }

        if next_states.is_empty() {
            return (vec![], vec![])
        }

        (next_states, actions)
    }

    fn new_with_pieces(pieces: [Position; 2 * Board::N_PIECES]) -> Self {
        Self { turn: 0, pieces }
    }

    fn possible_moves_with_path(&self, piece: Position) -> [Position; Board::BOARD_SIZE] {
        let mut result = [INVALID_POSITION; Board::BOARD_SIZE];
        let mut queue = vec![piece];

        result[piece as usize] = piece;

        while let Some(pos) = queue.pop() {
            for direction in 0..6 {
                let mut cp = pos;
                let mut steps = 0; // the distance to pos when hopping not started, or the steps remaing when hopping started
                let mut hopping_started = false;

                loop {
                    cp = Board::ADJ_MATRIX[cp as usize][direction];
                    if cp == INVALID_POSITION {
                        break
                    }

                    match (cp != piece && self.has_piece(cp), hopping_started, steps) {
                        (true, true, _) => break, // encounter obstacle, stop
                        (true, false, _) => hopping_started = true, // start hopping
                        (false, true, 0) => { // hopping succeed
                            if result[cp as usize] != INVALID_POSITION { // can be reached by another (shorter) path
                                break
                            }
                            queue.push(cp);
                            result[cp as usize] = pos;
                            break
                        }
                        (false, true, _) => steps -= 1, // hopping continue
                        (false, false, _) => steps += 1, // continue to move
                    }
                }
            }
        }

        // append single moves
        for direction in 0..6 {
            let next = Board::ADJ_MATRIX[piece as usize][direction];
            if next == INVALID_POSITION || self.has_piece(next) {
                continue
            }

            result[next as usize] = piece; // overide if exist because this must be the shortest
        }

        result
    }

    fn heuristic(&self) -> f64 { // the default value, higher is better for p1
        let mut p1_score = self.p1_score();
        if p1_score == Board::MIN_SCORE {
            p1_score = 0
        };

        let mut p2_score = self.p2_score();
        if p2_score == Board::MIN_SCORE {
            p2_score = 0
        };

        p2_score as f64 - p1_score as f64
    }
}


static mut RANDOM: u32 = 39393;

fn get_random_number() -> u32 {
    unsafe {
        RANDOM ^= RANDOM << 13;
        RANDOM ^= RANDOM >> 17;
        RANDOM ^= RANDOM << 5;
        RANDOM
    }
}

fn get_random_float() -> f64 {
    get_random_number() as f64 / core::u32::MAX as f64
}

fn random_shuffle<T>(x: &mut [T]) {
    for i in 0..x.len()-1 {
        let j = get_random_number() as usize % (x.len() - i - 1);
        x.swap(i, i+j+1);
    }
}

fn softmax(x: &mut [f64], temp: f64) {
    x.iter_mut().for_each(|v| *v /= temp);
    let m = x.iter().map(|v| ordered_float::OrderedFloat(*v)).max().unwrap().into_inner();
    let s: f64 = x.iter().map(|v| (*v - m).exp()).sum();
    x.iter_mut().for_each(|v| *v = (*v - m - s.ln()).exp());
}

fn sample_categorical(probs: impl Iterator<Item=f64>) -> usize {
    let mut rand = get_random_float();
    for (i, p) in probs.enumerate() {
        if rand < p as _ {
            return i
        } else {
            rand -= p;
        }
    }
    unreachable!()
}
