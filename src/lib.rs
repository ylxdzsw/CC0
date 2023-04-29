#![allow(clippy::missing_safety_doc)]
#![feature(vec_into_raw_parts)]

use serde_json::{json, Value as JsonValue, Value::Null as JsonNull};

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
// pub mod mcts;


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
pub unsafe extern fn new_small_game() -> *mut game::Game {
    Box::leak(Box::new(game::Game::new(&board::SMALL_BOARD)))
}

#[no_mangle]
pub unsafe extern fn new_standard_game() -> *mut game::Game {
    Box::leak(Box::new(game::Game::new(&board::STANDARD_BOARD)))
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
pub unsafe extern fn alphabeta(game: *mut game::Game, depth: usize) {
    let game = &*game;
    let (_next_state, action) = alphabeta::alphabeta(game, depth);
    write_json_buffer(&json!({
        "from": action.from,
        "to": action.to,
        "path": action.path
    }));
}

#[no_mangle]
pub unsafe extern fn greedy(game: *mut game::Game, temp: f64) {
    let game = &*game;
    let (_next_state, action) = greedy::greedy(game, temp);
    write_json_buffer(&json!({
        "from": action.from,
        "to": action.to,
        "path": action.path
    }));
}

