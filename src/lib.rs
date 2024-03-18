#![allow(clippy::missing_safety_doc)]
#![feature(vec_into_raw_parts)]

use std::collections::BTreeMap;

use serde_json::{json, Value as JsonValue};

type Position = u8; // huge board (rank 6) have 253 slots, u8 is just perfect.

#[no_mangle]
pub static INVALID_POSITION: Position = Position::MAX;

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


pub mod board;
pub mod game;
pub mod alphabeta;
pub mod greedy;
pub mod mcts;


#[no_mangle]
static mut JSON_BUFFER: [usize; 3] = [0, 0, 0];

// write to the json buffer. The client need to call free_json_buffer after reading it.
unsafe fn write_json_buffer(value: &JsonValue) {
    let raw_parts = serde_json::to_vec(value).unwrap().into_raw_parts();
    JSON_BUFFER = [raw_parts.0 as _, raw_parts.1 as _, raw_parts.2 as _];
}

// read from the json buffer AND free it.
unsafe fn read_json_buffer() -> serde_json::Result<JsonValue> {
    let [ptr, len, capacity] = JSON_BUFFER;
    let buffer = Vec::from_raw_parts(ptr as *mut u8, len as _, capacity as _);
    let json = serde_json::from_slice(&buffer);
    json
}

#[no_mangle]
unsafe extern fn alloc_json_buffer(byte_length: u32) {
    let (ptr, len, capacity) = Vec::<u8>::with_capacity(byte_length as _).into_raw_parts();
    JSON_BUFFER = [ptr as _, len as _, capacity as _];
}

#[no_mangle]
unsafe extern fn free_json_buffer() {
    let (ptr, len, capacity) = (JSON_BUFFER[0] as *mut u8, JSON_BUFFER[1] as _, JSON_BUFFER[2] as _);
    let _ = Vec::from_raw_parts(ptr, len, capacity);
}

#[no_mangle]
unsafe extern fn set_random_seed(seed: u32) {
    RANDOM = seed;
}

#[no_mangle]
pub unsafe extern fn new_tiny_game() -> *mut game::Game {
    Box::leak(Box::new(game::Game::new(&board::TINY_BOARD)))
}

#[no_mangle]
pub unsafe extern fn new_small_game() -> *mut game::Game {
    Box::leak(Box::new(game::Game::new(&board::SMALL_BOARD)))
}

#[no_mangle]
pub unsafe extern fn new_standard_game() -> *mut game::Game {
    Box::leak(Box::new(game::Game::new(&board::STANDARD_BOARD)))
}

#[no_mangle]
pub unsafe extern fn new_large_game() -> *mut game::Game {
    Box::leak(Box::new(game::Game::new(&board::LARGE_BOARD)))
}

#[no_mangle]
pub unsafe extern fn new_huge_game() -> *mut game::Game {
    Box::leak(Box::new(game::Game::new(&board::HUGE_BOARD)))
}

#[no_mangle]
pub unsafe extern fn new_tiny_plus_game() -> *mut game::Game {
    Box::leak(Box::new(game::Game::new(&board::TINY_PLUS_BOARD)))
}

#[no_mangle]
pub unsafe extern fn new_small_plus_game() -> *mut game::Game {
    Box::leak(Box::new(game::Game::new(&board::SMALL_PLUS_BOARD)))
}

#[no_mangle]
pub unsafe extern fn new_standard_plus_game() -> *mut game::Game {
    Box::leak(Box::new(game::Game::new(&board::STANDARD_PLUS_BOARD)))
}

#[no_mangle]
pub unsafe extern fn new_large_plus_game() -> *mut game::Game {
    Box::leak(Box::new(game::Game::new(&board::LARGE_PLUS_BOARD)))
}

#[no_mangle]
pub unsafe extern fn new_huge_plus_game() -> *mut game::Game {
    Box::leak(Box::new(game::Game::new(&board::HUGE_PLUS_BOARD)))
}

#[no_mangle]
pub unsafe extern fn free_game(game: *mut game::Game) {
    let _ = Box::from_raw(game);
}

#[no_mangle]
pub unsafe extern fn game_board_info(game: *mut game::Game) {
    let game = &*game;
    write_json_buffer(&json!({
        "n_pieces": game.board.n_pieces,
        "board_size": game.board.board_size,
    }))
}

#[no_mangle]
pub unsafe extern fn game_is_p1_moving_next(game: *mut game::Game) -> bool {
    let game = &*game;
    game.is_p1_moving_next()
}

#[no_mangle]
pub unsafe extern fn game_is_p2_moving_next(game: *mut game::Game) -> bool {
    let game = &*game;
    game.is_p2_moving_next()
}

#[no_mangle]
pub unsafe extern fn game_p1_pieces(game: *mut game::Game) {
    let game = &*game;
    write_json_buffer(&json!(game.p1_pieces_slice()))
}

#[no_mangle]
pub unsafe extern fn game_p2_pieces(game: *mut game::Game) {
    let game = &*game;
    write_json_buffer(&json!(game.p2_pieces_slice()))
}

#[no_mangle]
pub unsafe extern fn game_get_status(game: *mut game::Game) -> u8 {
    let game = &*game;
    if game.expand(false).0.is_empty() {
        match game.p1_distance().cmp(&game.p2_distance()) {
            std::cmp::Ordering::Less => 1,
            std::cmp::Ordering::Greater => 2,
            std::cmp::Ordering::Equal => 3,
        }
    } else {
        0
    }
}

// a score used by model2
#[no_mangle]
pub unsafe extern fn game_distance_diff_score(game: *mut game::Game) -> f64 {
    let game = &*game;
    let d1 = game.p1_distance();
    let d2 = game.p2_distance();
    if d1 == d2 {
        return 0.
    }

    let diff = d1 as f64 - d2 as f64;
    return diff.signum() * (1. + diff.abs().sqrt()) / 2.
}

#[no_mangle]
pub unsafe extern fn game_move_to(game: *mut game::Game, from: u8, to: u8) {
    let game = &mut *game;
    *game = game.move_to(from, to);
}

#[no_mangle]
pub unsafe extern fn game_possible_moves_with_path(game: *mut game::Game, piece: u8) {
    let game = &*game;
    let moves = game.possible_moves_with_path(piece);
    write_json_buffer(&json!(moves));
}

#[no_mangle]
pub unsafe extern fn game_turn(game: *mut game::Game) -> usize {
    let game = &*game;
    game.turn
}

#[no_mangle]
pub unsafe extern fn game_expand(game: *mut game::Game) {
    let game = &*game;
    let (next_states, _) = game.expand(false);
    let next_state_keys = next_states.into_iter().map(|next_state| next_state.key()).collect::<Vec<_>>();
    write_json_buffer(&json!(next_state_keys));
}

#[no_mangle]
pub unsafe extern fn game_key(game: *mut game::Game) {
    let game = &mut *game;
    write_json_buffer(&json!(game.key()));
}

#[no_mangle]
pub unsafe extern fn game_load_key(game: *mut game::Game) {
    let game = &mut *game;
    let key = read_json_buffer().unwrap();
    *game = game::Game::from_key(game, &key.as_array().unwrap().iter().map(|x| x.as_u64().unwrap() as u8).collect::<Vec<_>>());
}

#[no_mangle]
pub unsafe extern fn alphabeta(game: *mut game::Game, depth: usize) {
    let game = &*game;
    let (_next_state, action) = alphabeta::alphabeta(game, depth);
    write_json_buffer(&json!([action.0, action.1]));
}

#[no_mangle]
pub unsafe extern fn alphabeta_poll(game: *mut game::Game, depth: usize, forward_only: bool, mut sess: *mut BTreeMap<Vec<u8>, f64>) -> *mut BTreeMap<Vec<u8>, f64> {
    let game = &*game;
    let first_call = sess.is_null();

    if first_call {
        sess = Box::leak(Box::new(BTreeMap::new()));
    }
    let map = &mut *sess;

    if !first_call {
        let data = read_json_buffer().unwrap();
        for x in data.as_array().unwrap().iter() {
            let x = x.as_array().unwrap();
            let key = x[0].as_array().unwrap().iter().map(|x| x.as_u64().unwrap() as u8).collect::<Vec<_>>();
            let value = x[1].as_f64().unwrap();
            map.insert(key, value);
        }
    }

    match alphabeta::alphabeta_poll(game, depth, forward_only, map) {
        Ok((_next_state, action)) => {
            write_json_buffer(&json!([action.0, action.1]));
            let _ = Box::from_raw(sess);
            std::ptr::null_mut()
        },
        Err(keys) => {
            write_json_buffer(&json!(keys));
            sess
        }
    }
}

#[no_mangle]
pub unsafe extern fn greedy(game: *mut game::Game, temp: f64) {
    let game = &*game;
    let (_next_state, action) = greedy::greedy(game, temp);
    write_json_buffer(&json!([action.0, action.1]));
}

#[no_mangle]
pub unsafe extern fn greedy_poll(game: *mut game::Game, temp: f64, forward_only: bool, mut sess: *mut BTreeMap<Vec<u8>, f64>) -> *mut BTreeMap<Vec<u8>, f64> {
    let game = &*game;
    let first_call = sess.is_null();

    if first_call {
        sess = Box::leak(Box::new(BTreeMap::new()));
    }
    let map = &mut *sess;

    if !first_call {
        let data = read_json_buffer().unwrap();
        for x in data.as_array().unwrap().iter() {
            let x = x.as_array().unwrap();
            let key = x[0].as_array().unwrap().iter().map(|x| x.as_u64().unwrap() as u8).collect::<Vec<_>>();
            let value = x[1].as_f64().unwrap();
            map.insert(key, value);
        }
    }

    match greedy::greedy_poll(game, temp, forward_only, map) {
        Ok((_next_state, action)) => {
            write_json_buffer(&json!([action.0, action.1]));
            let _ = Box::from_raw(sess);
            std::ptr::null_mut()
        },
        Err(keys) => {
            write_json_buffer(&json!(keys));
            sess
        }
    }
}

#[no_mangle]
pub unsafe extern fn mcts(game: *mut game::Game, iterations: usize) {
    let game = &*game;
    let (_next_state, action) = mcts::mcts(game, iterations);
    write_json_buffer(&json!([action.0, action.1]));
}

#[no_mangle]
pub unsafe extern fn mcts_poll(game: *mut game::Game, iterations: usize, forward_only: bool, mut sess: *mut (mcts::Node, BTreeMap<Vec<u8>, f64>)) -> *mut (mcts::Node, BTreeMap<Vec<u8>, f64>) {
    let game = &*game;
    let first_call = sess.is_null();

    if first_call {
        sess = Box::leak(Box::new(mcts::new_session(game.clone())));
    }
    let (root, map) = &mut *sess;

    if !first_call {
        let data = read_json_buffer().unwrap();
        for x in data.as_array().unwrap().iter() {
            let x = x.as_array().unwrap();
            let key = x[0].as_array().unwrap().iter().map(|x| x.as_u64().unwrap() as u8).collect::<Vec<_>>();
            let value = x[1].as_f64().unwrap();
            map.insert(key, value);
        }
    }

    match mcts::mcts_poll(game, iterations, forward_only, (root, map)) {
        Ok((_next_state, action)) => {
            write_json_buffer(&json!([action.0, action.1]));
            let _ = Box::from_raw(sess);
            std::ptr::null_mut()
        },
        Err(keys) => {
            write_json_buffer(&json!(keys));
            sess
        }
    }
}

// a pure math function which is somehow tedieous to implement in js
#[no_mangle]
pub unsafe extern fn softmax_expectation(temp: f64, invert: bool) -> f64 {
    let data = read_json_buffer().unwrap();
    let data = data.as_array().unwrap();
    let data = data.iter().map(|x| x.as_f64().unwrap()).collect::<Vec<_>>();
    let mut prob = data.clone();
    if invert {
        prob.iter_mut().for_each(|x| *x = 1.0 - *x);
    }
    softmax(&mut prob, temp);
    data.iter().zip(prob.iter()).map(|(x, y)| x * y).sum::<f64>()
}
