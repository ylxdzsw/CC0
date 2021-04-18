#[cfg(target_arch="wasm32")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

type Position = u8; // huge board (rank 6) have 253 slots, u8 is just perfect.
const INVALID_POSITION: Position = Position::MAX;

mod board;
mod game;
mod mcts;

#[no_mangle]
pub unsafe extern fn alloc_memory(byte_size: u64) -> *mut u8 {
    vec![0u8; byte_size as _].leak() as *const _ as _
}

#[no_mangle]
pub unsafe extern fn free_memory(ptr: *mut u8, byte_size: u64) {
    Vec::from_raw_parts(ptr, byte_size as _, byte_size as _);
}

#[no_mangle]
pub unsafe extern fn new_standard_game() -> *mut game::Game<board::StandardBoard> {
    Box::leak(Box::new(game::Game::<board::StandardBoard>::new()))
}

/// returned list is encoded as [INVALID_POSITION, pieces_pos_1, pieces_move_1, pieces_move_2, INVALID_POSITION, pieces_pos_1, ...]
#[no_mangle]
pub unsafe extern fn possible_moves(game: *mut game::Game<board::StandardBoard>, out: *mut *mut Position, length: *mut u64) {
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

/// 1: first player won, 2: second player won, 3: tie, 0: unfinished.
#[no_mangle]
pub unsafe extern fn do_move(game: *mut game::Game<board::StandardBoard>, from: Position, to: Position) -> i8 {
    let game = &mut *game;
    game.move_with_role_change(from, to);
    if game.finished() {
        let (w1, w2) = game.score();
        match w1.cmp(&w2) {
            std::cmp::Ordering::Greater => 1,
            std::cmp::Ordering::Less => 2,
            std::cmp::Ordering::Equal => 3,
        }
    } else {
        0
    }
}
