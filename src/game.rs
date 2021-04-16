use core::usize;

use crate::{Position, INVALID_POSITION, board::Board};

#[derive(Debug, Eq, PartialEq)]
enum Player { First, Second }
impl Player {
    fn change(&mut self) {
        match *self {
            Player::First => *self = Player::Second,
            Player::Second => *self = Player::First
        }
    }
}

struct Piece {
    owner: Player,
    position: Position
}

impl Piece {
    fn new(owner: Player, position: Position) -> Self {
        Self { owner, position }
    }
}

pub struct Game<B: Board> {
    phantom: core::marker::PhantomData<B>,
    pieces: Vec<Piece>,
    board: Vec<Option<usize>>, // `pieces` is the primary state, `board` is just an position -> piece index
    player: Player, // current player. the First player plays first.
    total_moves: usize
}

impl<B: Board> Game<B> {
    pub fn new() -> Self {
        let mut game = Game {
            phantom: core::marker::PhantomData,
            pieces: vec![],
            board: vec![None; B::board_size()],
            player: Player::First,
            total_moves: 0
        };
        game.reset();
        game

    }

    pub fn reset(&mut self) {
        self.pieces.clear();
        for &i in B::base_ids().0 {
            self.board[i as usize] = Some(self.pieces.len());
            self.pieces.push(Piece::new(Player::First, i));
        }
        for &i in B::base_ids().1 {
            self.board[i as usize] = Some(self.pieces.len());
            self.pieces.push(Piece::new(Player::Second, i))
        }
    }

    /// find possible moves of p by BFS. result[i] = j means p can move to i via j. -1 means impossible move.
    pub fn possible_moves(&self, pos: Position) -> Vec<Position> {
        let p = &self.pieces[self.board[pos as usize].unwrap()];
        let mut result = vec![INVALID_POSITION; B::board_size()];
        let mut queue = vec![p.position];
        result[p.position as usize] = p.position;

        while let Some(pos) = queue.pop() {
            for direction in 0..6 {
                let mut cp = pos;
                let mut steps = 0; // the distance to pos when hopping not started, or the steps remaing when hopping started
                let mut hopping_started = false;

                loop {
                    cp = B::adj(cp)[direction];
                    if cp == INVALID_POSITION {
                        break
                    }

                    match (&self.board[cp as usize], hopping_started, steps) {
                        (Some(_), true, _) => break, // encounter obstacle, stop
                        (Some(_), false, _) => hopping_started = true, // start hopping
                        (None, true, 0) => { // hopping succeed
                            if result[cp as usize] != INVALID_POSITION { // can be reached by another (shorter) path
                                break
                            }
                            queue.push(cp);
                            result[cp as usize] = pos;
                            break
                        }
                        (None, true, _) => steps -= 1, // hopping continue
                        (None, false, _) => steps += 1, // continue to move
                    }
                }
            }
        }

        // append single moves
        for direction in 0..6 {
            let next = B::adj(p.position)[direction];
            if next == INVALID_POSITION || self.board[next as usize].is_some() {
                continue
            }

            result[next as usize] = p.position; // overide if exist because this must be the shortest
        }

        result
    }

    pub fn move_with_role_change(&mut self, from: Position, to: Position) {
        assert!(self.board[to as usize].is_none()); // target location empty

        let pid = self.board[from as usize].take().unwrap(); // the board updated once here
        let piece = &mut self.pieces[pid];
        assert_eq!(piece.owner, self.player); // it is the piece of the current player

        piece.position = to;
        self.player.change();
        self.total_moves += 1;
        self.board[to as usize] = Some(pid);
    }

    pub fn finished(&self) -> bool {
        let (w1, w2) = self.score();
        w1 == B::n_pieces() || w2 == B::n_pieces()
    }

    // evaluate the pieces that are in the opponents base
    pub fn score(&self) -> (usize, usize) {
        let mut w1 = 0;
        let mut w2 = 0;
        for p in &self.pieces {
            if p.owner == Player::First && B::base_ids().1.contains(&p.position) { w1 += 1; }
            if p.owner == Player::Second && B::base_ids().0.contains(&p.position) { w2 += 1; }
        }
        (w1, w2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_test_1() {
        let mut game = Game::<crate::board::StandardBoard>::new();
        let mut possible_moves;

        possible_moves = game.possible_moves(8);
        for &p in &[16, 17] { assert!(possible_moves[p] != INVALID_POSITION) }
        game.move_with_role_change(8, 16);

        possible_moves = game.possible_moves(114);
        for &p in &[105, 106] { assert!(possible_moves[p] != INVALID_POSITION) }
        game.move_with_role_change(114, 106);

        possible_moves = game.possible_moves(5);
        for &p in &[8, 18, 39] { assert!(possible_moves[p] != INVALID_POSITION) }
        game.move_with_role_change(5, 39);

        possible_moves = game.possible_moves(117);
        for &p in &[83, 114] { assert!(possible_moves[p] != INVALID_POSITION) }
        game.move_with_role_change(117, 83);

        possible_moves = game.possible_moves(0);
        for &p in &[5, 14, 18, 60] { assert!(possible_moves[p] != INVALID_POSITION) }
        game.move_with_role_change(0, 60);

        possible_moves = game.possible_moves(115);
        for &p in &[58, 62, 102, 104, 108, 117] { assert!(possible_moves[p] != INVALID_POSITION) }
    }
}
