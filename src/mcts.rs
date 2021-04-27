use crate::{INVALID_POSITION, Position, game::{Status, Game, Player}};
use alloc::vec::Vec;
use ordered_float::OrderedFloat;

fn get_random_number() -> u32 {
    static mut RANDOM: u32 = 39393;

    unsafe {
        RANDOM ^= RANDOM << 13;
        RANDOM ^= RANDOM >> 17;
        RANDOM ^= RANDOM << 5;
        RANDOM
    }
}

fn get_random_float() -> f32 {
    get_random_number() as f32 / core::u32::MAX as f32
}

fn sample_categorical(probs: impl Iterator<Item=f64>) -> usize {
    let mut rand = get_random_float();
    for (i, p) in probs.enumerate() {
        if rand < p as _ {
            return i
        } else {
            rand -= p as f32;
        }
    }
    unreachable!()
}

fn uniform_random_choice<T>(x: &[T]) -> &T {
    &x[get_random_number() as usize % x.len()]
}

fn softmax(x: &mut [f64], temp: f64) {
    x.iter_mut().for_each(|v| *v = *v / temp);
    let m = x.iter().map(|v| OrderedFloat(*v)).max().unwrap().into_inner();
    let s: f64 = x.iter().map(|v| (*v - m).exp()).sum();
    x.iter_mut().for_each(|v| *v = (*v - m - s.ln()).exp());
}

// A parent node holds the ownership of it children. For simplicity and unsafe-free, the child node do not hold
// reference to its parent. Therefore the traveling methods must be recursive and save the path on stack.
struct Node {
    action: (Position, Position),
    player: Player, // who plays the action
    children: Vec<Node>,
    n_visits: u64,
    q: f64,
    p: f64
}

impl Node {
    fn new(action: (Position, Position), player: Player, p: f64) -> Self {
        Self { action, player, children: vec![], n_visits: 0, q: 0., p }
    }

    /// playout a game and update the path, return the leaf_value for the parent
    // Note about leaf_value: the leaf_value applied to a node should be large if the action of the node is favorable for the last player (it is the `player` of the node)
    fn playout_and_update_recursive(&mut self, game: &mut Game, policy: Option<&PolicyCallback>) -> f64 {
        if self.is_leaf() {
            let leaf_value = match game.status() {
                Status::Winner(winner) => {
                    if winner == self.player { 1. } else { -1. }
                }
                Status::Tie => 0.,
                Status::Unfinished => {
                    if let Some(policy) = policy { // use policy value to estimate
                        let (pick_p, move_p, value) = policy(game); // the game's next player is the other player (not the node's player). The value returned by the NN is about the game state, which describe how likely the next player can win.
                        self.expand(game, &pick_p, &move_p);
                        -value
                    } else { // pure mcts, uniformly expand and use random rollout until end to estimate value
                        self.expand_uniform(game);
                        self.rollout(game)
                    }
                }
            };

            self.update(leaf_value);
            return -leaf_value; // invert the sign because the player changes
        }

        // choose an action greedily
        let child = self.select_chlid();
        let (from, to) = child.action;
        game.move_with_role_change(from, to);
        let leaf_value = child.playout_and_update_recursive(game, policy);
        self.update(leaf_value);
        -leaf_value
    }

    fn is_leaf(&self) -> bool { self.children.is_empty() }

    fn select_chlid(&mut self) -> &mut Node {
        let pvisit = self.n_visits;
        self.children.iter_mut().max_by_key(|x| OrderedFloat(x.puct(pvisit))).expect("selecting from no children")
    }

    // unlike UCB in most MCTS, AlphaZero uses a variant called PUCT.
    fn puct(&self, pvisit: u64) -> f64 {
        const c_puct: f64 = 4.; // unlike c in UCB which takes 1.4, c_puct usually set around 4 or 5.
        let u = c_puct * self.p * (pvisit as f64).sqrt() / (1. + self.n_visits as f64);
        self.q + u
    }

    fn update(&mut self, leaf_value: f64) {
        self.n_visits += 1;
        self.q += (leaf_value - self.q) / self.n_visits as f64
    }

    /// pick_p is a list similar to game.pieces. Unmovable pieces and the pieces of the opponenets shoule have be 0.
    /// move_p is an array of (2 * n_pieces) * board_size. Eachline is the probabilities of moving targets. Illegle moves should have 0.
    fn expand(&mut self, game: &Game, pick_p: &[f64], move_p: &[f64]) {
        let pieces = game.get_pieces();
        let board_size = game.board_size();

        for piece_id in 0..2*game.n_pieces() {
            if pick_p[piece_id] <= f64::EPSILON {
                continue
            }

            let from = pieces[piece_id].position;
            for to in 0..board_size {
                if move_p[piece_id * board_size + to] <= f64::EPSILON {
                    continue
                }

                let action = (from as _, to as _);
                let prior = pick_p[piece_id] * move_p[piece_id * board_size + to];
                self.children.push(Node::new(action, self.player.the_other(), prior))
            }
        }
    }

    fn expand_uniform(&mut self, game: &Game) {
        let all_valid_moves: Vec<_> = game.all_pieces_and_possible_moves_of_current_player().into_iter()
            .filter(|(_, moves)| !moves.is_empty())
            .collect();

        let n_movable_pieces = all_valid_moves.len();

        for (from, moves) in all_valid_moves {
            let n_targets = moves.len();
            for to in moves {
                self.children.push(Node::new((from, to), self.player.the_other(), 1. / n_targets as f64 / n_movable_pieces as f64))
            }
        }
    }

    fn rollout(&self, game: &mut Game) -> f64 {
        while !game.status().finished() {
            let all_valid_moves: Vec<_> = game.all_pieces_and_possible_moves_of_current_player().into_iter()
                .filter(|(_, moves)| !moves.is_empty())
                .collect();
            let (from, moves) = uniform_random_choice(&all_valid_moves);
            let to = uniform_random_choice(moves);
            game.move_with_role_change(*from, *to);
        }
        match game.status() {
            Status::Winner(winner) => {
                if winner == self.player { 1. } else { -1. }
            }
            Status::Tie => 0.,
            Status::Unfinished => unreachable!()
        }
    }
}

type PolicyCallback = dyn Fn(&Game) -> (Vec<f64>, Vec<f64>, f64);

pub struct Tree {
    root: Node,
    policy: Option<Box<PolicyCallback>> // pick p, move p, value
}

impl Tree {
    pub fn new(policy: Option<Box<PolicyCallback>>) -> Self {
        Self { root: Node::new((INVALID_POSITION, INVALID_POSITION), Player::First, 1.), policy }
    }

    pub fn playout(&mut self, game: &Game, ntimes: usize) {
        self.root.player = game.last_player(); // the children of root will be the one play next
        for n in 0..ntimes {
            self.root.playout_and_update_recursive(&mut game.clone(), self.policy.as_deref());
        }
    }

    pub fn get_move_probs(&self, temp: f64) -> Vec<(Position, Position, f64)> { // from, to, prob
        debug_assert!(!self.root.children.is_empty());
        let mut visits: Vec<f64> = self.root.children.iter().map(|node| node.n_visits as _).collect();
        for v in visits.iter_mut() {
            *v = (*v + 1e-10).ln()
        }
        softmax(&mut visits, temp);
        self.root.children.iter().zip(visits).map(|(node, prob)| (node.action.0, node.action.1, prob)).collect()
    }

    // Choose a subtree and step into next state.
    // This is used in self-play to reuse the searched subtree.
    // note that in evaluation we should create new trees as the subtree is search assuming the opponent uses the same strategy.
    pub fn chroot(&mut self, action: (Position, Position)) {
        let pos = self.root.children.iter().position(|n| n.action == action).expect("cannot find the action in root children");
        let child = self.root.children.swap_remove(pos);
        self.root = child;
    }

    // sample an action using the root visit counts.
    // exploration_prob: 0 in inference, 0.1 in self-play
    // temperature: 1e-3 in inference, 0.1 in self-play
    pub fn sample_action(&mut self, game: &Game, exploration_prob: f32, temperature: f64) -> (Position, Position) {
        let acts = self.get_move_probs(temperature);
        // I don't understand why the paper introduces the Dirichlet distribution. It seems to me that it is quivalent to the following implementation.
        let sampled_act = if get_random_float() < exploration_prob {
            uniform_random_choice(&acts)
        } else {
            let sampled_index = sample_categorical(acts.iter().map(|(_, _, p)| *p));
            &acts[sampled_index]
        };
        (sampled_act.0, sampled_act.1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_soft_max() {
        let mut x = [1., 2., 3., 4., 1., 2., 3.];
        softmax(&mut x, 1.);
        for (a, b) in x.iter().zip([0.02364, 0.06426, 0.17468, 0.47483, 0.02364, 0.06426, 0.17468].iter()) {
            assert!((a - b).abs() < 1e-5)
        }
    }

    #[test] #[ignore]
    fn test_mcts_playout() {
        let start_time = std::time::Instant::now();
        let mut mcts = Tree::new(None);
        let mut game = Game::new(&crate::board::STANDARD_BOARD);
        let mut total_playout = 0;

        loop {
            let n_playout = 2000 - mcts.root.n_visits;
            mcts.playout(&game, n_playout as _);
            total_playout += n_playout;
            let (from, to) = mcts.sample_action(&game, 0., 1e-3);
            println!("{:?} move {} to {}", game.next_player(), from, to);
            game.move_with_role_change(from, to);
            match game.status() {
                Status::Winner(winner) => {
                    println!("{:?} won!", winner);
                    break
                }
                Status::Tie => {
                    println!("tie!");
                    break
                }
                Status::Unfinished => {}
            }
            mcts.chroot((from, to));
            // mcts = Tree::new(None);
        }

        let elapsed_time = start_time.elapsed().as_millis();
        println!("total playout: {}, elasped: {}, playout/ms: {}", total_playout, elapsed_time, total_playout as f64 / elapsed_time as f64)
    }
}
