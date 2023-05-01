use std::collections::BTreeMap;

use crate::{game::{Game, Action}, random_shuffle};

pub struct Node {
    game: Game,
    action: Action,
    children: Vec<Node>,
    n_visits: usize,
    priori: f64, // p1 winning prob. We will invert it during selection score calculation
    value: f64 // similarly, it's p1 winning rate
}

impl Node {
    fn expand(&mut self, score_map: &BTreeMap<Vec<u8>, f64>) -> Result<bool, Vec<Vec<u8>>> { // the bool indicates if the node is leaf
        let (next_states, actions) = self.game.expand(true);

        if next_states.is_empty() {
            return Ok(true)
        }

        let no_prioris: Vec<_> = next_states.iter().map(|g| g.key()).filter(|k| !score_map.contains_key(k)).collect();
        if !no_prioris.is_empty() {
            return Err(no_prioris)
        }

        let mut zipped: Vec<_> = next_states.into_iter().zip(actions.into_iter()).collect();
        random_shuffle(&mut zipped);

        self.children = zipped.into_iter().map(|(game, action)| {
            let score = score_map[&game.key()];
            Node {
                game, action,
                children: Vec::new(),
                n_visits: 0,
                priori: score,
                value: 0.5
            }
        }).collect();

        Ok(false)
    }

    fn select(&mut self) -> &mut Node {
        self.children.iter_mut().max_by_key(|child| {
            let (q, p) = if self.game.is_p1_moving_next() {
                (child.value, child.priori)
            } else {
                (1. - child.value, 1. - child.priori)
            };

            let puct = q + 1.41 * p * (self.n_visits as f64).sqrt() / (1. + child.n_visits as f64);
            ordered_float::OrderedFloat(puct)
        }).unwrap()
    }

    fn playout(&mut self, score_map: &BTreeMap<Vec<u8>, f64>) -> Result<f64, Vec<Vec<u8>>> {
        if self.children.is_empty() {
            self.n_visits += 1;

            let leaf_value = if self.expand(score_map)? { // is leaf
                match self.game.p1_distance().cmp(&self.game.p2_distance()) {
                    std::cmp::Ordering::Less => 1.,
                    std::cmp::Ordering::Greater => 0.,
                    std::cmp::Ordering::Equal => 0.,
                }
            } else {
                // we have several options here
                // 1. recursively expand to leaf
                // 2. play to end with policy
                // 3. play to end with heuristic
                // 4. estimate the value with policy
                // 5. estimate the vlaue with heuristic
                match self.game.p1_distance().cmp(&self.game.p2_distance()) {
                    std::cmp::Ordering::Less => 1.,
                    std::cmp::Ordering::Greater => 0.,
                    std::cmp::Ordering::Equal => if self.game.is_p1_moving_next() {
                        1.
                    } else {
                        0.
                    }
                }
            };

            self.value = leaf_value;
            return Ok(leaf_value)
        }

        let leaf_value = self.select().playout(score_map)?;

        self.n_visits += 1;
        self.value += (leaf_value - self.value) / self.n_visits as f64;
        Ok(leaf_value)
    }
}

pub fn new_session(game: Game) -> (Node, BTreeMap<Vec<u8>, f64>) {
    let action = Action { from: 255, to: 255, path: vec![] };
    (Node { game, action, children: Vec::new(), n_visits: 0, priori: 0.5, value: 0.5 }, BTreeMap::new())
}

pub fn mcts_poll(_game: &Game, itertions: usize, sess: (&mut Node, &BTreeMap<Vec<u8>, f64>)) -> Result<(Game, Action), Vec<Vec<u8>>> {
    let (root, score_map) = sess;

    while root.n_visits < itertions {
        root.playout(score_map)?;
    }

    Ok(root.children.iter().max_by_key(|child| child.n_visits).map(|child| (child.game.clone(), child.action.clone())).unwrap())
}

pub fn mcts(game: &Game, itertions: usize) -> (Game, Action) {
    let (mut root, mut score_map) = new_session(game.clone());

    loop {
        match mcts_poll(game, itertions, (&mut root, &score_map)) {
            Ok((game, action)) => return (game, action),
            Err(no_prioris) => {
                for key in no_prioris {
                    let baseline = 2. * game.board.n_pieces as f64;
                    let heuristic = Game::from_key(&game, &key).heuristic();
                    let value = if heuristic >= baseline {
                        1.0
                    } else if heuristic <= -baseline {
                        0.0
                    } else {
                        0.5 + heuristic / (2. * baseline)
                    };

                    score_map.insert(key, value);
                }
            }
        }
    }
}
