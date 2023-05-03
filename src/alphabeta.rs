use std::collections::BTreeMap;

use crate::{game::{Game, Action}, random_shuffle};

fn _alphabeta(game: &Game, remaining_depth: usize, alpha: f64, beta: f64) -> f64 {
    if remaining_depth <= 0 {
        return game.heuristic()
    }

    let (next_states, _) = game.expand(false);
    if next_states.is_empty() {
        return game.heuristic()
    }

    if game.is_p1_moving_next() {
        let mut value = alpha;

        for next_state in next_states {
            value = value.max(_alphabeta(&next_state, remaining_depth - 1, value, beta));
            if value >= beta {
                break
            }
        }
        value
    } else {
        let mut value = beta;

        for next_state in next_states {
            value = value.min(_alphabeta(&next_state, remaining_depth - 1, alpha, value));
            if value <= alpha {
                break
            }
        }
        value
    }
}

pub fn alphabeta(game: &Game, depth: usize) -> (Game, Action) {
    let (next_states, actions) = game.expand(true);
    let mut zipped: Vec<_> = next_states.into_iter().zip(actions.into_iter()).collect();
    random_shuffle(&mut zipped);

    let values = zipped.iter().map(|(next_state, _)| _alphabeta(next_state, depth - 1, std::f64::NEG_INFINITY, std::f64::INFINITY)).collect::<Vec<_>>();
    let i = if game.is_p1_moving_next() {
        values.iter().enumerate().max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap()).unwrap().0
    } else {
        values.iter().enumerate().min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap()).unwrap().0
    };

    zipped.swap_remove(i)
}

fn _alphabeta_poll(game: &Game, remaining_depth: usize, forward_only: bool, alpha: f64, beta: f64, score_map: &BTreeMap<Vec<u8>, f64>) -> Result<f64, Vec<Vec<u8>>> {
    if remaining_depth <= 0 {
        return score_map.get(&game.key()).copied().ok_or_else(|| vec![game.key().clone()])
    }

    let (next_states, _) = if forward_only {
        game.expand_forward_only(false)
    } else {
        game.expand(false)
    };
    if next_states.is_empty() {
        return score_map.get(&game.key()).copied().ok_or_else(|| vec![game.key().clone()])
    }

    if game.is_p1_moving_next() {
        let mut value = alpha;

        for next_state in next_states {
            value = value.max(_alphabeta_poll(&next_state, remaining_depth - 1, forward_only, value, beta, score_map)?);
            if value >= beta {
                break
            }
        }
        Ok(value)
    } else {
        let mut value = beta;

        for next_state in next_states {
            value = value.min(_alphabeta_poll(&next_state, remaining_depth - 1, forward_only, alpha, value, score_map)?);
            if value <= alpha {
                break
            }
        }
        Ok(value)
    }
}

pub fn alphabeta_poll(game: &Game, depth: usize, forward_only: bool, score_map: &BTreeMap<Vec<u8>, f64>) -> Result<(Game, Action), Vec<Vec<u8>>> {
    let (next_states, actions) = if forward_only {
        game.expand_forward_only(true)
    } else {
        game.expand(true)
    };

    let mut zipped: Vec<_> = next_states.into_iter().zip(actions.into_iter()).collect();
    random_shuffle(&mut zipped);

    let values = zipped.iter().map(|(next_state, _)| _alphabeta_poll(next_state, depth - 1, forward_only, std::f64::NEG_INFINITY, std::f64::INFINITY, score_map)).collect::<Vec<_>>();
    let i = if game.is_p1_moving_next() {
        values.iter().enumerate().max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap()).unwrap().0
    } else {
        values.iter().enumerate().min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap()).unwrap().0
    };

    Ok(zipped.swap_remove(i))
}
