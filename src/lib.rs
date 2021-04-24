// the calculation of sqrt needs std
// #![no_std]

#[macro_use]
extern crate alloc;

use alloc::vec::Vec;
use alloc::boxed::Box;

type Position = u8; // huge board (rank 6) have 253 slots, u8 is just perfect.

#[no_mangle]
pub static INVALID_POSITION: Position = Position::MAX;

pub mod board;
pub mod game;
pub mod mcts;

#[no_mangle]
pub unsafe extern fn alloc_memory(byte_size: u64) -> *mut u8 {
    vec![0u8; byte_size as _].leak() as *const _ as _
}

#[no_mangle]
pub unsafe extern fn free_memory(ptr: *mut u8, byte_size: u64) {
    Vec::from_raw_parts(ptr, byte_size as _, byte_size as _);
}

#[no_mangle]
pub unsafe extern fn new_standard_game() -> *mut game::Game {
    Box::leak(Box::new(game::Game::new(&board::STANDARD_BOARD)))
}

#[no_mangle]
pub unsafe extern fn new_small_game() -> *mut game::Game {
    Box::leak(Box::new(game::Game::new(&board::SMALL_BOARD)))
}

#[no_mangle]
pub unsafe extern fn get_board_size(game: *mut game::Game) -> u64 {
    (*game).board_size() as _
}

#[no_mangle]
pub unsafe extern fn get_n_pieces(game: *mut game::Game) -> u64 {
    (*game).n_pieces() as _
}

/// returned list is encoded as [INVALID_POSITION, pieces_pos_1, pieces_move_1, pieces_move_2, INVALID_POSITION, pieces_pos_1, ...]
#[no_mangle]
pub unsafe extern fn get_possible_moves(game: *mut game::Game, out: *mut *mut Position, length: *mut u64) {
    let possible_moves = (*game).all_pieces_and_possible_moves_of_current_player();

    let mut encoded = vec![];
    for (piece_pos, mut moves) in possible_moves {
        encoded.push(INVALID_POSITION);
        encoded.push(piece_pos);
        encoded.append(&mut moves);
    }

    *length = encoded.len() as _;
    *out = encoded.leak() as *const _ as _;
}

#[no_mangle]
pub unsafe extern fn do_move(game: *mut game::Game, from: Position, to: Position) {
    (*game).move_with_role_change(from, to);
}

/// 1: first player won, 2: second player won, 3: tie, 0: unfinished.
#[no_mangle]
pub unsafe extern fn get_status(game: *mut game::Game) -> u8 {
    match (*game).status() {
        game::Status::Winner(game::Player::First) => 1,
        game::Status::Winner(game::Player::Second) => 2,
        game::Status::Tie => 3,
        game::Status::Unfinished => 0
    }
}

/// dump game state.
/// 1st byte: n_pieces
/// 2nd byte: current player. 1 for the first player, 2 for the second.
/// following 2*`n_pieces` bytes: the position of each pieces, with the first half belongs to the first player.
#[no_mangle]
pub unsafe extern fn dump(game: *mut game::Game, out: *mut *mut Position, length: *mut u64) {
    let game = &mut *game;
    let mut encoded = vec![];

    encoded.push(game.n_pieces() as _);
    encoded.push(match game.next_player() {
        game::Player::First => 1,
        game::Player::Second => 2
    });

    for player in &[game::Player::First, game::Player::Second] {
        for piece in game.get_pieces() {
            if &piece.owner == player {
                encoded.push(piece.position)
            }
        }
    }

    *length = encoded.len() as _;
    *out = encoded.leak() as *const _ as _;
}
