#![feature(core_intrinsics)]
#![allow(clippy::missing_safety_doc)]

use std::vec::Vec;
use std::boxed::Box;

type Position = u8; // huge board (rank 6) have 253 slots, u8 is just perfect.
type EncodedAction = u32;

fn encode_action(from: Position, to: Position) -> EncodedAction {
    ((from as EncodedAction) << 8) + (to as EncodedAction)
}

fn decode_action(encoded_action: EncodedAction) -> (Position, Position) {
    ((encoded_action >> 8) as _, encoded_action as _)
}

#[no_mangle]
pub static INVALID_POSITION: Position = Position::MAX;

pub mod board;
pub mod game;
pub mod mcts;

#[no_mangle]
pub unsafe extern fn alloc_memory(byte_size: u32) -> *mut u8 {
    Vec::<u8>::with_capacity(byte_size as _).leak() as *const _ as _
}

#[no_mangle]
pub unsafe extern fn free_memory(ptr: *mut u8, byte_size: u32) {
    // Note the second argument is the length. We set it equals to capacity, which may causing droping uninitialized memory if we were dealing with element types that are not u8.
    Vec::from_raw_parts(ptr, byte_size as _, byte_size as _);
}

#[no_mangle]
pub unsafe extern fn set_random_seed(seed: u32) {
    mcts::set_random_seed(seed)
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
pub unsafe extern fn get_board_size(game: *mut game::Game) -> u32 {
    (*game).board_size() as _
}

#[no_mangle]
pub unsafe extern fn get_n_pieces(game: *mut game::Game) -> u32 {
    (*game).n_pieces() as _
}

/// the returned length is only used for deallocation. The `out` length is the same as board size and not explicitly returned.
#[no_mangle]
pub unsafe extern fn possible_moves_with_path(game: *mut game::Game, pos: Position, out: *mut *mut Position, length: *mut u32) {
    let possible_moves_with_path = (*game).possible_moves_with_path(pos);
    *length = possible_moves_with_path.capacity() as _;
    *out = possible_moves_with_path.leak() as *const _ as _;
}

/// returned list is encoded as [INVALID_POSITION, pieces_pos_1, pieces_move_1, pieces_move_2, INVALID_POSITION, pieces_pos_1, ..., INVALID_POSITION, INVALID_POSITION]
#[no_mangle]
pub unsafe extern fn all_possible_moves(game: *mut game::Game, out: *mut *mut Position, length: *mut u32) {
    let possible_moves = (*game).movable_pieces_and_possible_moves_of_current_player();

    let mut encoded = vec![];
    for (piece_pos, mut moves) in possible_moves {
        encoded.push(INVALID_POSITION);
        encoded.push(piece_pos);
        encoded.append(&mut moves);
    }

    // terminated with two INVALID_POSITIONs
    encoded.push(INVALID_POSITION);
    encoded.push(INVALID_POSITION);

    *length = encoded.capacity() as _;
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
/// the returned length is only used for deallocation. The `out` length is determined by 2 + 2 * n_pieces
#[allow(clippy::vec_init_then_push)]
#[no_mangle]
pub unsafe extern fn dump(game: *mut game::Game, out: *mut *mut Position, length: *mut u32) {
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

    *length = encoded.capacity() as _;
    *out = encoded.leak() as *const _ as _;
}

#[no_mangle]
pub unsafe extern fn destroy_game(game: *mut game::Game) {
    Box::from_raw(game);
}

/// policy is represented by an array of n_pieces * board_size. policy[i * board_size + j] is the probability of move
/// the i-th (in "dump" order) piece of the next player to the position j. The array should be normalized with invalid
/// actions having probability 0.
#[no_mangle]
pub unsafe extern fn new_mcts(policy_cfun: extern fn (*mut game::Game, *mut f32, *mut f32)) -> *mut mcts::Tree {
    let policy_value_callback = Box::new(move |game: &game::Game| {
        let mut prior = vec![0.0; game.n_pieces() * game.board_size()]; // TODO: uninited?
        let mut value = f32::NAN; // invalid value to check if `policy_fun` runs correctly

        policy_cfun(game as *const _ as _, prior.as_mut_ptr(), &mut value as _);

        if value.is_nan() { // `policy_fun` did not exit correctly, possibly raised exception as the result of Ctrl-C
            panic!("policy callback did not run correctly"); // or should abort?
        }

        let self_pieces: Vec<_> = game.get_pieces().iter().filter(|p| p.owner == game.next_player()).collect();
        let board_size = game.board_size();

        let mut action_probs = vec![];
        for (from, moves) in game.movable_pieces_and_possible_moves_of_current_player() {
            let i = self_pieces.iter().position(|x| x.position == from).expect("movable piece that not belongs to the player");
            for to in moves {
                action_probs.push((from, to, prior[i * board_size + to as usize]))
            }
        }

        (action_probs, value)
    });
    Box::leak(Box::new(mcts::Tree::new(Some(policy_value_callback))))
}

#[no_mangle]
pub unsafe extern fn new_mcts_pure() -> *mut mcts::Tree {
    Box::leak(Box::new(mcts::Tree::new(None)))
}

#[no_mangle]
pub unsafe extern fn mcts_playout(mcts: *mut mcts::Tree, game: *mut game::Game, ntimes: u32) {
    (*mcts).playout(&*game, ntimes as _)
}

#[no_mangle]
pub unsafe extern fn start_try_playout(mcts: *mut mcts::Tree, game: *mut *mut game::Game, ntimes: u32) -> *mut mcts::TryPlayoutContinuation {
    // the game passed in is unmodified, but the game pointer is pointed to the played game (or null if finished)
    if let Some(cont) = (*mcts).try_playout(&**game, ntimes as _) {
        *game = cont.game;
        Box::leak(Box::new(cont)) as *mut _
    } else {
        *game = core::ptr::null_mut();
        core::ptr::null_mut()
    }
}

#[no_mangle]
pub unsafe extern fn continue_try_playout(cont: *mut *mut mcts::TryPlayoutContinuation, game: *mut *mut game::Game, prior: *mut f32, value: f32) {
    let self_pieces: Vec<_> = (**game).get_pieces().iter().filter(|p| p.owner == (**game).next_player()).collect();
    let board_size = (**game).board_size();

    let mut action_probs = vec![];
    for (from, moves) in (**game).movable_pieces_and_possible_moves_of_current_player() {
        let i = self_pieces.iter().position(|x| x.position == from).expect("movable piece that not belongs to the player");
        for to in moves {
            action_probs.push((from, to, prior.add(i * board_size + to as usize).read()))
        }
    }
    if let Some(new_cont) = (Box::from_raw(*cont).cont)(action_probs, value) {
        *game = new_cont.game;
        *cont = Box::leak(Box::new(new_cont));
    } else {
        *game = core::ptr::null_mut();
        *cont = core::ptr::null_mut();
    }
}

#[no_mangle]
pub unsafe extern fn mcts_get_action_probs(mcts: *mut mcts::Tree, temp: f32, actions: *mut EncodedAction, probs: *mut f32, length: *mut u32) {
    let action_probs = (*mcts).get_action_probs(temp);
    *length = action_probs.len() as _;

    if actions.is_null() || probs.is_null() { // this call just queries the length for allocation.
        return
    }

    let actions = core::slice::from_raw_parts_mut(actions, action_probs.len());
    let probs = core::slice::from_raw_parts_mut(probs, action_probs.len());

    for (i, (from, to, p)) in action_probs.into_iter().enumerate() {
        actions[i] = encode_action(from, to);
        probs[i] = p
    }
}

#[no_mangle]
pub unsafe extern fn mcts_sample_action(mcts: *mut mcts::Tree, exploration_prob: f32, temperature: f32) -> EncodedAction {
    let (from, to) = (*mcts).sample_action(exploration_prob, temperature);
    encode_action(from, to)
}

#[no_mangle]
pub unsafe extern fn mcts_chroot(mcts: *mut mcts::Tree, encoded_action: EncodedAction) {
    let action = decode_action(encoded_action);
    (*mcts).chroot(action)
}

#[no_mangle]
pub unsafe extern fn mcts_total_visits(mcts: *mut mcts::Tree) -> u32 {
    (*mcts).total_visits()
}

#[no_mangle]
pub unsafe extern fn mcts_root_value(mcts: *mut mcts::Tree) -> f32 {
    (*mcts).root_value()
}


#[no_mangle]
pub unsafe extern fn destroy_mcts(mcts: *mut mcts::Tree) {
    Box::from_raw(mcts);
}
