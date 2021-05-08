use alloc::vec::Vec;

use crate::{INVALID_POSITION, Position, board::Board};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Player { First, Second }
impl Player {
    pub fn change(&mut self) {
        match *self {
            Player::First => *self = Player::Second,
            Player::Second => *self = Player::First
        }
    }

    pub fn the_other(&self) -> Self {
        match *self {
            Player::First => Player::Second,
            Player::Second => Player::First
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Status { Winner(Player), Tie, Unfinished }

impl Status {
    pub fn finished(&self) -> bool { *self != Status::Unfinished }
}

#[derive(Debug, Clone)]
pub struct Piece {
    pub owner: Player,
    pub position: Position
}

impl Piece {
    fn new(owner: Player, position: Position) -> Self {
        Self { owner, position }
    }
}

#[derive(Clone)]
pub struct Game {
    board_def: &'static dyn Board,
    pieces: Vec<Piece>,
    pindex: Vec<Option<usize>>, // position -> piece index
    player: Player, // the player that will play next. the First player plays first.
    total_moves: usize
}

impl Game {
    pub fn new(board_def: &'static dyn Board) -> Self {
        let mut pieces = vec![];
        let mut pindex = vec![None; board_def.board_size()];

        for &i in board_def.base_ids().0 {
            pindex[i as usize] = Some(pieces.len());
            pieces.push(Piece::new(Player::First, i));
        }
        for &i in board_def.base_ids().1 {
            pindex[i as usize] = Some(pieces.len());
            pieces.push(Piece::new(Player::Second, i))
        }

        debug_assert_eq!(pieces.len(), 2 * board_def.n_pieces());

        Game { board_def, pieces, pindex, player: Player::First, total_moves: 0 }
    }

    pub fn reset(&mut self) {
        *self = Game::new(self.board_def)
    }

    pub fn rank(&self) -> usize { self.board_def.rank() }
    pub fn board_size(&self) -> usize { self.board_def.board_size() }
    pub fn n_pieces(&self) -> usize { self.board_def.n_pieces() }
    pub fn turn_limit(&self) -> usize { self.board_def.turn_limit() }
    pub fn adj(&self, center: Position) -> &'static [Position] { self.board_def.adj(center) }
    pub fn base_ids(&self) -> (&'static [Position], &'static [Position]) { self.board_def.base_ids()}

    pub fn next_player(&self) -> Player {
        self.player
    }

    pub fn last_player(&self) -> Player {
        self.player.the_other()
    }

    pub fn get_pieces(&self) -> &[Piece] {
        &self.pieces
    }

    /// find possible moves of p by BFS. result[i] = j means p can move to i via j. INVALID_POSITION means unreachable.
    pub fn possible_moves_with_path(&self, pos: Position) -> Vec<Position> {
        let moving_piece_id = self.pindex[pos as usize].unwrap();
        let moving_piece_pos = self.pieces[moving_piece_id].position;
        let mut result = vec![INVALID_POSITION; self.board_def.board_size()];
        let mut queue = vec![moving_piece_pos];
        result[moving_piece_pos as usize] = moving_piece_pos;

        while let Some(pos) = queue.pop() {
            for direction in 0..6 {
                let mut cp = pos;
                let mut steps = 0; // the distance to pos when hopping not started, or the steps remaing when hopping started
                let mut hopping_started = false;

                loop {
                    cp = self.board_def.adj(cp)[direction];
                    if cp == INVALID_POSITION {
                        break
                    }

                    match (&self.pindex[cp as usize], hopping_started, steps) {
                        (Some(pid), true, _) if *pid != moving_piece_id => break, // encounter obstacle, stop
                        (Some(pid), false, _) if *pid != moving_piece_id => hopping_started = true, // start hopping
                        (_, true, 0) => { // hopping succeed
                            if result[cp as usize] != INVALID_POSITION { // can be reached by another (shorter) path
                                break
                            }
                            queue.push(cp);
                            result[cp as usize] = pos;
                            break
                        }
                        (_, true, _) => steps -= 1, // hopping continue
                        (_, false, _) => steps += 1, // continue to move
                    }
                }
            }
        }

        // append single moves
        for direction in 0..6 {
            let next = self.board_def.adj(moving_piece_pos)[direction];
            if next == INVALID_POSITION || self.pindex[next as usize].is_some() {
                continue
            }

            result[next as usize] = moving_piece_pos; // overide if exist because this must be the shortest
        }

        result
    }

    pub fn possible_moves(&self, pos: Position) -> Vec<Position> {
        self.possible_moves_with_path(pos).into_iter()
            .enumerate()
            .filter(|&(i, p)| p != INVALID_POSITION && pos != i as _)
            .map(|(i, _)| i as _)
            .collect()
    }

    pub fn movable_pieces_and_possible_moves_of_current_player(&self) -> Vec<(Position, Vec<Position>)> {
        self.pieces.iter()
            .filter(|p| p.owner == self.player)
            .map(|p| (p.position, self.possible_moves(p.position)))
            .filter(|(_, moves)| !moves.is_empty())
            .collect()
    }

    pub fn move_with_role_change(&mut self, from: Position, to: Position) {
        debug_assert!(self.pindex[to as usize].is_none()); // target location empty

        let pid = self.pindex[from as usize].take().unwrap(); // the board updated once here
        let piece = &mut self.pieces[pid];
        debug_assert_eq!(piece.owner, self.player); // it is the piece of the current player

        piece.position = to;
        self.player.change();
        self.total_moves += 1;
        self.pindex[to as usize] = Some(pid);
    }

    // the winning rule is slightly changed a bit for effective training:
    // 1. After the first turn of each player, the player whose base gets filled (with both player's pieces) loss immediately
    // 2. When the turn limits reached (defaults to 5 * n_pieces), the player whose base have less pieces win.
    // This is due to the following observations:
    // 1. If we use the simplest rule, that moving all pieces to the opponent's base, pure MCTS never reach an end and the training cannot start.
    // 2. The second method I tried is to force an end when turn limit reached. The player that put most pieces to the opponent's base win.
    //   - It looks compatiable to the original rule. However, after training the agents become "defensive" that when a player is going to lose, it will return a piece back to its own base so that the opponent cannot move in, force the game to end in tie.
    pub fn status(&self) -> Status {
        if self.total_moves < 2 {
            return Status::Unfinished
        }

        let (mut n_pieces_in_first_player_base, mut n_pieces_in_second_player_base) = (0, 0);
        for p in &self.pieces {
            if self.board_def.base_ids().0.contains(&p.position) { n_pieces_in_first_player_base += 1; }
            if self.board_def.base_ids().1.contains(&p.position) { n_pieces_in_second_player_base += 1; }
        }

        if n_pieces_in_first_player_base == self.board_def.n_pieces() {
            return Status::Winner(Player::Second)
        }

        if n_pieces_in_second_player_base == self.board_def.n_pieces() {
            return Status::Winner(Player::First)
        }

        if self.total_moves >= 2 * self.board_def.turn_limit() {
            match n_pieces_in_first_player_base.cmp(&n_pieces_in_second_player_base) {
                core::cmp::Ordering::Less => Status::Winner(Player::First),
                core::cmp::Ordering::Equal => Status::Tie,
                core::cmp::Ordering::Greater => Status::Winner(Player::Second)
            }
        } else {
            Status::Unfinished
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_test_1() {
        let mut game = Game::new(&crate::board::STANDARD_BOARD);
        let mut possible_moves;

        possible_moves = game.possible_moves_with_path(8);
        for &p in &[16, 17] { assert!(possible_moves[p] != INVALID_POSITION) }
        game.move_with_role_change(8, 16);

        possible_moves = game.possible_moves_with_path(114);
        for &p in &[105, 106] { assert!(possible_moves[p] != INVALID_POSITION) }
        game.move_with_role_change(114, 106);

        possible_moves = game.possible_moves_with_path(5);
        for &p in &[8, 18, 39] { assert!(possible_moves[p] != INVALID_POSITION) }
        game.move_with_role_change(5, 39);

        possible_moves = game.possible_moves_with_path(117);
        for &p in &[83, 114] { assert!(possible_moves[p] != INVALID_POSITION) }
        game.move_with_role_change(117, 83);

        possible_moves = game.possible_moves_with_path(0);
        for &p in &[5, 14, 18, 60] { assert!(possible_moves[p] != INVALID_POSITION) }
        game.move_with_role_change(0, 60);

        possible_moves = game.possible_moves_with_path(115);
        for &p in &[58, 62, 102, 104, 108, 117] { assert!(possible_moves[p] != INVALID_POSITION) }
    }
}
