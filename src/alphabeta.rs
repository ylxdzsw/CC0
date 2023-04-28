use crate::{game::{Game, Action}, random_shuffle};

// higher is better for p1
fn heuristic(game: &Game) -> f64 {
    let mut p1_dist = game.p1_distance();
    if p1_dist <= game.board.min_distance {
        p1_dist = 0 // enlarge the wining gap
    }

    let mut p2_dist = game.p2_distance();
    if p2_dist <= game.board.min_distance {
        p2_dist = 0
    }

    p2_dist as f64 - p1_dist as f64
}

fn _alphabeta(game: &Game, remaining_depth: usize, alpha: f64, beta: f64) -> f64 {
    if remaining_depth <= 0 {
        return heuristic(game)
    }

    let (next_states, _) = game.expand(false);
    if next_states.is_empty() {
        return heuristic(game)
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
