use std::vec::Vec;

use crate::{INVALID_POSITION, Position, board::Board};

#[derive(Clone)]
pub struct Action {
    pub from: Position,
    pub to: Position,
    pub path: Vec<Position>
}

#[derive(Clone)]
pub struct Game {
    pub board: &'static Board,
    pub turn: usize, // Initial state (no players has moved and player 1 is about to move next) has turn = 0.
    pub pieces: Vec<Position>, // first half are the pieces of p1 and second half are p2. Both sorted seperately.
}

impl Game {
    pub fn new(board: &'static Board) -> Self {
        Game { board, turn: 0, pieces: board.starting_pieces() }
    }

    pub fn is_p1_moving_next(&self) -> bool {
        self.turn % 2 == 0
    }

    pub fn is_p2_moving_next(&self) -> bool {
        self.turn % 2 == 1
    }

    pub fn p1_pieces_slice(&self) -> &[u8] {
        &self.pieces[..self.board.n_pieces]
    }

    pub fn p1_pieces_slice_mut(&mut self) -> &mut [u8] {
        &mut self.pieces[..self.board.n_pieces]
    }

    pub fn p2_pieces_slice(&self) -> &[u8] {
        &self.pieces[self.board.n_pieces..]
    }

    pub fn p2_pieces_slice_mut(&mut self) -> &mut [u8] {
        &mut self.pieces[self.board.n_pieces..]
    }

    pub fn has_piece(&self, piece: Position) -> bool {
        self.pieces.contains(&piece) // or two binary searches?
    }

    pub fn move_to(&self, from: Position, to: Position) -> Self {
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

    pub fn p1_distance(&self) -> u64 {
        self.p1_pieces_slice().iter().map(|&p| self.board.p1_distance_map[p as usize]).sum()
    }

    pub fn p2_distance(&self) -> u64 {
        self.p2_pieces_slice().iter().map(|&p| self.board.p2_distance_map[p as usize]).sum()
    }

    pub fn expand(&self, record_actions: bool) -> (Vec<Game>, Vec<Action>) {
        // early finish
        if self.p1_distance() == self.board.min_distance || self.p2_distance() == self.board.min_distance {
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

            for dest in paths.iter().enumerate().filter(|&(dest, from)| *from != INVALID_POSITION && dest as u8 != piece).map(|(dest, _)| dest as Position) {
                next_states.push(self.move_to(piece, dest));
                if record_actions {
                    actions.push(Action { from: piece, to: dest, path: paths.clone() });
                }
            }
        }

        if next_states.is_empty() {
            return (vec![], vec![])
        }

        (next_states, actions)
    }

    pub fn clone_with_pieces(&self, pieces: &[Position]) -> Self {
        Self { pieces: pieces.to_vec(), ..self.clone() }
    }

    pub fn possible_moves_with_path(&self, piece: Position) -> Vec<Position> {
        let mut result = vec![INVALID_POSITION; self.board.board_size];
        let mut queue = vec![piece];

        result[piece as usize] = piece;

        while let Some(pos) = queue.pop() {
            for direction in 0..6 {
                let mut cp = pos;
                let mut steps = 0; // the distance to pos when hopping not started, or the steps remaing when hopping started
                let mut hopping_started = false;

                loop {
                    cp = self.board.ajd_matrix[cp as usize][direction];
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
            let next = self.board.ajd_matrix[piece as usize][direction];
            if next == INVALID_POSITION || self.has_piece(next) {
                continue
            }

            result[next as usize] = piece; // overide if exist because this must be the shortest
        }

        result
    }

    // higher is better for p1
    pub fn heuristic(&self) -> f64 {
        let mut p1_dist = self.p1_distance();
        if p1_dist <= self.board.min_distance {
            p1_dist = 0 // enlarge the wining gap
        }

        let mut p2_dist = self.p2_distance();
        if p2_dist <= self.board.min_distance {
            p2_dist = 0
        }

        p2_dist as f64 - p1_dist as f64
    }

    pub fn key(&self) -> Vec<u8> {
        let mut result = vec![self.turn as u8];
        result.extend_from_slice(&self.pieces);
        result
    }

    pub fn from_key(proto: &Game, key: &[u8]) -> Game {
        let mut game = Game::new(&proto.board);
        game.turn = key[0] as _;
        game.pieces = key[1..].to_vec();
        game
    }
}

#[cfg(test)]
mod tests {
    use crate::board::SMALL_BOARD;

    use super::*;

    #[test]
    fn game_test_1() {
        let mut game = Game::new(&SMALL_BOARD);

        game.p1_pieces_slice_mut().copy_from_slice(&[3, 4, 5, 9, 10, 11]);

        let possible_moves = game.possible_moves_with_path(4);
        println!("{:?}", possible_moves);
    }
}


