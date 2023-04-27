use std::{sync::{RwLock, atomic::{AtomicU32, Ordering, AtomicBool}}, collections::BTreeMap, io::{Write, Read}};

const USAGE: &str = "
Usage: super_mcts playout [count=10000] [threads=auto]
       super_mcts analyze
";

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let args: Vec<_> = args.iter().map(|x| &x[..]).collect();

    let mut db = DB::new();
    if std::path::Path::new("result.bin").exists() {
        db.load();
    }

    match args[..] {
        ["playout"] => playout(10000, &mut db, 0),
        ["playout", count] => playout(count.parse().unwrap(), &mut db, 0),
        ["playout", count, threads] => playout(count.parse().unwrap(), &mut db, threads.parse().unwrap()),
        _ => eprint!("{}", USAGE)
    }
}

struct DB {
    tree: RwLock<BTreeMap<[u8; 2 * BOARD_N_PIECES], (AtomicU32, AtomicU32)>>
}

impl DB {
    fn new() -> Self {
        Self {
            tree: RwLock::new(BTreeMap::new())
        }
    }

    fn query(&self, state: &[u8; 2 * BOARD_N_PIECES]) -> (usize, usize) {
        let tree = self.tree.read().unwrap();
        tree.get(state)
            .map(|(p1win, p2win)| (p1win.load(Ordering::Relaxed) as usize, p2win.load(Ordering::Relaxed) as usize))
            .unwrap_or((0, 0))
    }

    fn record_status(&self, state: &[u8; 2 * BOARD_N_PIECES], status: Status) {
        // TODO: bloom filter?

        // try to not write lock the tree
        {
            let tree = self.tree.read().unwrap();
            if let Some((p1win, p2win)) = tree.get(state) {
                match status {
                    Status::P1Win => p1win.fetch_add(1, Ordering::Relaxed),
                    Status::P2Win => p2win.fetch_add(1, Ordering::Relaxed),
                    _ => unreachable!()
                };
                return
            }
        }

        // write lock the tree
        {
            let mut tree = self.tree.write().unwrap();
            let (p1win, p2win) = tree.entry(*state).or_insert((AtomicU32::new(0), AtomicU32::new(0)));
            match status {
                Status::P1Win => p1win.fetch_add(1, Ordering::Relaxed),
                Status::P2Win => p2win.fetch_add(1, Ordering::Relaxed),
                _ => unreachable!()
            };
        }
    }

    fn dump(&self) {
        let tree = self.tree.read().unwrap();
        let result_file = std::fs::File::create("result.bin").unwrap();
        let mut result_file_writer = std::io::BufWriter::new(result_file);
        for (state, (p1win, p2win)) in tree.iter() {
            let (p1win, p2win) = (p1win.load(Ordering::Relaxed), p2win.load(Ordering::Relaxed));
            if p1win + p2win >= 10 {
                result_file_writer.write_all(state).unwrap();
                result_file_writer.write_all(&p1win.to_le_bytes()).unwrap();
                result_file_writer.write_all(&p2win.to_le_bytes()).unwrap();
            }
        }
        result_file_writer.flush().unwrap();
    }

    fn load(&self) {
        let mut tree = self.tree.write().unwrap();
        let result_file = std::fs::File::open("result.bin").unwrap();
        let mut result_file_reader = std::io::BufReader::new(result_file);
        loop {
            let mut state = [0; 2 * BOARD_N_PIECES];
            let mut p1win = [0; 4];
            let mut p2win = [0; 4];
            if result_file_reader.read_exact(&mut state).is_err() {
                break;
            }
            result_file_reader.read_exact(&mut p1win).unwrap();
            result_file_reader.read_exact(&mut p2win).unwrap();
            let (p1win, p2win) = (u32::from_le_bytes(p1win), u32::from_le_bytes(p2win));
            tree.insert(state, (AtomicU32::new(p1win), AtomicU32::new(p2win)));
        }
    }
}

static STOP: AtomicBool = AtomicBool::new(false);

fn playout(count: usize, db: &DB, threads: usize) {
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
                    playout_game(Game::new(), db);
                }
            });
        }
    });

    db.dump();
}

fn playout_game(game: Game, db: &DB) -> Status {
    let (status, mut next_states) = game.eval();

    if status != Status::Pending {
        db.record_status(game.key(), status);
        return status;
    }

    random_shuffle(&mut next_states);

    // randomly choose a move in several turns
    if game.turn() <= 2 * BOARD_N_PIECES && get_random_float() < 0.5 {
        let final_status = playout_game(next_states[0], db);
        db.record_status(game.key(), final_status);
        return final_status;
    }

    let history: Vec<_> = next_states.iter().map(|g| {
        db.query(g.key())
    }).collect();
    let c = 1.41;
    let pvisit: usize = history.iter().map(|(a, b)| a + b).sum(); // not entirely accurate
    let log_pvisit = (pvisit as f64).ln();
    let scores = history.iter().map(|(w, l)| {
        let mut w = *w as f64;
        let mut l = *l as f64;
        if game.is_p2_moving_next() {
            std::mem::swap(&mut w, &mut l);
        }
        let n = w + l;
        (w / n) + c * (log_pvisit / n).sqrt()
    }).collect::<Vec<_>>();
    let next_state = next_states.into_iter().zip(scores).max_by(|(_, a), (_, b)| a.total_cmp(b)).unwrap().0;
    let final_status = playout_game(next_state, db);
    db.record_status(game.key(), final_status);
    final_status
}

type Position = u8;

const INVALID_POSITION: Position = Position::MAX;

const BOARD_N_PIECES: usize = 10;
const BOARD_SIZE: usize = 121;
const BOARD_TURN_LIMIT: usize = 4 * BOARD_N_PIECES;
const BOARD_ADJ_MATRIX: &'static [[Position; 6]; BOARD_SIZE] = &[[1,2,255,255,255,255],[3,4,2,0,255,255],[4,5,255,255,0,1],[6,7,4,1,255,255],[7,8,5,2,1,3],[8,9,255,255,2,4],[14,15,7,3,255,255],[15,16,8,4,3,6],[16,17,9,5,4,7],[17,18,255,255,5,8],[255,23,11,255,255,255],[23,24,12,255,255,10],[24,25,13,255,255,11],[25,26,14,255,255,12],[26,27,15,6,255,13],[27,28,16,7,6,14],[28,29,17,8,7,15],[29,30,18,9,8,16],[30,31,19,255,9,17],[31,32,20,255,255,18],[32,33,21,255,255,19],[33,34,22,255,255,20],[34,255,255,255,255,21],[255,35,24,11,10,255],[35,36,25,12,11,23],[36,37,26,13,12,24],[37,38,27,14,13,25],[38,39,28,15,14,26],[39,40,29,16,15,27],[40,41,30,17,16,28],[41,42,31,18,17,29],[42,43,32,19,18,30],[43,44,33,20,19,31],[44,45,34,21,20,32],[45,255,255,22,21,33],[255,46,36,24,23,255],[46,47,37,25,24,35],[47,48,38,26,25,36],[48,49,39,27,26,37],[49,50,40,28,27,38],[50,51,41,29,28,39],[51,52,42,30,29,40],[52,53,43,31,30,41],[53,54,44,32,31,42],[54,55,45,33,32,43],[55,255,255,34,33,44],[255,56,47,36,35,255],[56,57,48,37,36,46],[57,58,49,38,37,47],[58,59,50,39,38,48],[59,60,51,40,39,49],[60,61,52,41,40,50],[61,62,53,42,41,51],[62,63,54,43,42,52],[63,64,55,44,43,53],[64,255,255,45,44,54],[65,66,57,47,46,255],[66,67,58,48,47,56],[67,68,59,49,48,57],[68,69,60,50,49,58],[69,70,61,51,50,59],[70,71,62,52,51,60],[71,72,63,53,52,61],[72,73,64,54,53,62],[73,74,255,55,54,63],[75,76,66,56,255,255],[76,77,67,57,56,65],[77,78,68,58,57,66],[78,79,69,59,58,67],[79,80,70,60,59,68],[80,81,71,61,60,69],[81,82,72,62,61,70],[82,83,73,63,62,71],[83,84,74,64,63,72],[84,85,255,255,64,73],[86,87,76,65,255,255],[87,88,77,66,65,75],[88,89,78,67,66,76],[89,90,79,68,67,77],[90,91,80,69,68,78],[91,92,81,70,69,79],[92,93,82,71,70,80],[93,94,83,72,71,81],[94,95,84,73,72,82],[95,96,85,74,73,83],[96,97,255,255,74,84],[98,99,87,75,255,255],[99,100,88,76,75,86],[100,101,89,77,76,87],[101,102,90,78,77,88],[102,103,91,79,78,89],[103,104,92,80,79,90],[104,105,93,81,80,91],[105,106,94,82,81,92],[106,107,95,83,82,93],[107,108,96,84,83,94],[108,109,97,85,84,95],[109,110,255,255,85,96],[255,255,99,86,255,255],[255,255,100,87,86,98],[255,255,101,88,87,99],[255,255,102,89,88,100],[255,111,103,90,89,101],[111,112,104,91,90,102],[112,113,105,92,91,103],[113,114,106,93,92,104],[114,255,107,94,93,105],[255,255,108,95,94,106],[255,255,109,96,95,107],[255,255,110,97,96,108],[255,255,255,255,97,109],[255,115,112,103,102,255],[115,116,113,104,103,111],[116,117,114,105,104,112],[117,255,255,106,105,113],[255,118,116,112,111,255],[118,119,117,113,112,115],[119,255,255,114,113,116],[255,120,119,116,115,255],[120,255,255,117,116,118],[255,255,255,119,118,255]];
const BOARD_P1_BASE: &'static [Position; BOARD_N_PIECES] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
const BOARD_P2_BASE: &'static [Position; BOARD_N_PIECES] = &[111, 112, 113, 114, 115, 116, 117, 118, 119, 120];
const STARTING_STATE: &'static [Position; 1 + 2 * BOARD_N_PIECES] = &[0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 111, 112, 113, 114, 115, 116, 117, 118, 119, 120];
const BOARD_P1_SCORE: &'static [usize; BOARD_SIZE] = &[0, 1, 1, 2, 2, 2, 3, 3, 3, 3, 8, 7, 6, 5, 4, 4, 4, 4, 4, 5, 6, 7, 8, 8, 7, 6, 5, 5, 5, 5, 5, 5, 6, 7, 8, 8, 7, 6, 6, 6, 6, 6, 6, 6, 7, 8, 8, 7, 7, 7, 7, 7, 7, 7, 7, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 13, 13, 13, 13, 14, 14, 14, 15, 15, 16];
const BOARD_P2_SCORE: &'static [usize; BOARD_SIZE] = &[16, 15, 15, 14, 14, 14, 13, 13, 13, 13, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 7, 7, 7, 7, 7, 7, 7, 7, 8, 8, 7, 6, 6, 6, 6, 6, 6, 6, 7, 8, 8, 7, 6, 5, 5, 5, 5, 5, 5, 6, 7, 8, 8, 7, 6, 5, 4, 4, 4, 4, 4, 5, 6, 7, 8, 3, 3, 3, 3, 2, 2, 2, 1, 1, 0];

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum Status { P1Win, P2Win, Pending }

#[derive(Clone, Copy, Default)]
#[repr(transparent)]
struct Game([u8; 1 + 2 * BOARD_N_PIECES]);

impl Game {
    fn new() -> Self {
        Self(*STARTING_STATE)
    }

    // Initial state (no players has moved and player 1 is about to move next) has turn = 0.
    fn turn(&self) -> usize {
        self.0[0] as usize
    }

    fn is_p1_moving_next(&self) -> bool {
        self.turn() % 2 == 0
    }

    fn is_p2_moving_next(&self) -> bool {
        self.turn() % 2 == 1
    }

    fn pieces_slice(&self) -> &[u8] {
        &self.0[1..]
    }

    fn p1_pieces_slice(&self) -> &[u8] {
        &self.0[1..=BOARD_N_PIECES]
    }

    fn p1_pieces_slice_mut(&mut self) -> &mut [u8] {
        &mut self.0[1..=BOARD_N_PIECES]
    }

    fn p2_pieces_slice(&self) -> &[u8] {
        &self.0[1+BOARD_N_PIECES..]
    }

    fn p2_pieces_slice_mut(&mut self) -> &mut [u8] {
        &mut self.0[1+BOARD_N_PIECES..]
    }

    fn has_piece(&self, piece: Position) -> bool {
        self.pieces_slice().binary_search(&piece).is_ok()
    }

    fn move_to(&self, from: Position, to: Position) -> Self {
        let mut result = self.clone();
        result.0[0] += 1;

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

    fn p1_score(&self) -> usize {
        self.p1_pieces_slice().iter().map(|&p| BOARD_P1_SCORE[p as usize]).sum()
    }

    fn p2_score(&self) -> usize {
        self.p2_pieces_slice().iter().map(|&p| BOARD_P2_SCORE[p as usize]).sum()
    }

    // eval gives the status of the game and the list of next states (potentially includes duplicates).
    fn eval(&self) -> (Status, Vec<Game>) {
        // early finish
        if self.is_p1_moving_next() && self.p2_pieces_slice() == BOARD_P1_BASE {
            return (Status::P2Win, vec![])
        }

        if self.is_p2_moving_next() && self.p1_pieces_slice() == BOARD_P2_BASE {
            return (Status::P1Win, vec![])
        }

        // force finish
        if self.turn() > BOARD_TURN_LIMIT {
            return match self.p1_score().cmp(&self.p2_score()) {
                std::cmp::Ordering::Greater => (Status::P2Win, vec![]),
                std::cmp::Ordering::Less => (Status::P1Win, vec![]),
                std::cmp::Ordering::Equal => (Status::P2Win, vec![]),
            }
        }

        let mut next_states = vec![];

        let moving_slice = if self.is_p1_moving_next() {
            self.p1_pieces_slice()
        } else {
            self.p2_pieces_slice()
        };

        for &piece in moving_slice {
            let paths = possible_moves_with_path(self, piece);

            for dest in paths.into_iter().enumerate().filter(|&(dest, from)| from != INVALID_POSITION && dest as u8 != piece).map(|(dest, _)| dest as Position) {
                next_states.push(self.move_to(piece, dest));
            }
        }

        let status = if next_states.is_empty() {
            if self.is_p1_moving_next() {
                Status::P2Win
            } else {
                Status::P1Win
            }
        } else {
            Status::Pending
        };

        (status, next_states)
    }

    // database key. Currently omit the turn.
    fn key(&self) -> &[u8; 2 * BOARD_N_PIECES] {
        self.0[1..].try_into().unwrap()
    }
}

fn possible_moves_with_path(game: &Game, piece: Position) -> [Position; BOARD_SIZE] {
    let mut result = [INVALID_POSITION; BOARD_SIZE];
    let mut queue = vec![piece];

    result[piece as usize] = piece;

    while let Some(pos) = queue.pop() {
        for direction in 0..6 {
            let mut cp = pos;
            let mut steps = 0; // the distance to pos when hopping not started, or the steps remaing when hopping started
            let mut hopping_started = false;

            loop {
                cp = BOARD_ADJ_MATRIX[cp as usize][direction];
                if cp == INVALID_POSITION {
                    break
                }

                match (cp != piece && game.has_piece(cp), hopping_started, steps) {
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
        let next = BOARD_ADJ_MATRIX[piece as usize][direction];
        if next == INVALID_POSITION || game.has_piece(next) {
            continue
        }

        result[next as usize] = piece; // overide if exist because this must be the shortest
    }

    result
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

fn get_random_float() -> f32 {
    get_random_number() as f32 / core::u32::MAX as f32
}

fn random_shuffle<T>(x: &mut [T]) {
    for i in 0..x.len()-1 {
        let j = get_random_number() as usize % (x.len() - i - 1);
        x.swap(i, i+j+1);
    }
}
