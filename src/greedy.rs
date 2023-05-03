use std::collections::BTreeMap;

use crate::{game::{Game, Action}, softmax, sample_categorical};

pub fn greedy(game: &Game, temp: f64) -> (Game, Action) {
    let (mut next_states, mut actions) = game.expand(true);

    if next_states.is_empty() {
        panic!("Game already ends!")
    }

    let mut values: Vec<_> = next_states.iter().map(|g| g.heuristic()).collect();
    if game.is_p2_moving_next() {
        values.iter_mut().for_each(|x| *x = -*x);
    }
    softmax(&mut values, temp);

    let i = sample_categorical(values.into_iter());

    (next_states.swap_remove(i), actions.swap_remove(i))
}

pub fn greedy_poll(game: &Game, temp: f64, forward_only: bool, score_map: &BTreeMap<Vec<u8>, f64>) -> Result<(Game, Action), Vec<Vec<u8>>> {
    let (mut next_states, mut actions) = if forward_only {
        game.expand_forward_only(true)
    } else {
        game.expand(true)
    };

    if next_states.is_empty() {
        panic!("Game already ends!")
    }

    let no_values: Vec<_> = next_states.iter().map(|g| g.key()).filter(|k| !score_map.contains_key(k)).collect();
    if !no_values.is_empty() {
        return Err(no_values)
    }

    let mut values: Vec<_> = next_states.iter().map(|g| score_map[&g.key()]).collect();
    if game.is_p2_moving_next() {
        values.iter_mut().for_each(|x| *x = 1. - *x);
    }
    softmax(&mut values, temp);

    let i = sample_categorical(values.into_iter());

    Ok((next_states.swap_remove(i), actions.swap_remove(i)))
}


