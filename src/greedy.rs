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

