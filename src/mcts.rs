use crate::{INVALID_POSITION, Position, game::{Status, Game, Player}};
use alloc::vec::Vec;
use alloc::boxed::Box;
use ordered_float::OrderedFloat;

static mut RANDOM: u32 = 39393;

pub fn set_random_seed(seed: u32) {
    unsafe { RANDOM = seed }
}

fn get_random_number() -> u32 {
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

fn sample_categorical(probs: impl Iterator<Item=f32>) -> usize {
    let mut rand = get_random_float();
    for (i, p) in probs.enumerate() {
        if rand < p as _ {
            return i
        } else {
            rand -= p;
        }
    }
    unreachable!()
}

fn uniform_random_choice<T>(x: &[T]) -> &T {
    &x[get_random_number() as usize % x.len()]
}

fn softmax(x: &mut [f32], temp: f32) {
    x.iter_mut().for_each(|v| *v /= temp);
    let m = x.iter().map(|v| OrderedFloat(*v)).max().unwrap().into_inner();
    let s: f32 = x.iter().map(|v| libm::expf(*v - m)).sum();
    x.iter_mut().for_each(|v| *v = libm::expf(*v - m - libm::logf(s)));
}

pub type Action = (Position, Position); // from, to
pub type ActionProb = (Position, Position, f32); // from, to, prob
pub type PolicyValueCallback = dyn Fn(&Game) -> (Vec<ActionProb>, f32);

// A parent node holds the ownership of it children. For simplicity and unsafe-free, the child node do not hold
// reference to its parent. Therefore the traveling methods must be recursive and save the path on stack.
struct Node {
    action: Action,
    player: Player, // who plays the action
    children: Vec<Node>,
    n_visits: u32,
    q: f32,
    p: f32
}

impl Node {
    fn new(action: Action, player: Player, p: f32) -> Self {
        Self { action, player, children: vec![], n_visits: 0, q: 0., p }
    }

    /// playout a game and update the path, return the leaf_value for the parent
    // Note about leaf_value: the leaf_value applied to a node should be large if the action of the node is favorable for the last player (it is the `player` of the node)
    fn playout_and_update_recursive(&mut self, mut game: Game, policy: Option<&PolicyValueCallback>) -> f32 {
        if self.is_leaf() {
            let leaf_value = match game.status() {
                Status::Winner(winner) => {
                    if winner == self.player { 1. } else { -1. }
                }
                Status::Tie => 0.,
                Status::Unfinished => {
                    if let Some(policy) = policy { // use policy value to estimate
                        let (action_probs, value) = policy(&game);
                        self.expand(&game, &action_probs);
                        -value // the game's next player is the other player (not the node's player). The value returned by the NN is about the game state, which describe how likely the next player (the player that is going to take action acroding to the policy) can win.
                    } else { // pure mcts, uniformly expand and use random rollout until end to estimate value
                        self.expand_uniform(&game);
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
    fn puct(&self, pvisit: u32) -> f32 {
        #[allow(non_upper_case_globals)]
        const c_puct: f32 = 2.;
        let u = c_puct * self.p * libm::sqrtf(pvisit as f32) / (1. + self.n_visits as f32);
        self.q + u
    }

    fn update(&mut self, leaf_value: f32) {
        self.n_visits += 1;
        self.q += (leaf_value - self.q) / self.n_visits as f32
    }

    fn expand(&mut self, game: &Game, action_probs: &[ActionProb]) {
        for &(from, to, p) in action_probs {
            self.children.push(Node::new((from, to), game.next_player(), p))
        }
    }

    fn expand_uniform(&mut self, game: &Game) {
        for (from, moves) in game.movable_pieces_and_possible_moves_of_current_player() {
            for to in moves {
                self.children.push(Node::new((from, to), game.next_player(), 0.))
            }
        }

        let p = 1. / self.children.len() as f32;
        for child in self.children.iter_mut() {
            child.p = p
        }
    }

    fn rollout(&self, mut game: Game) -> f32 {
        while !game.status().finished() {
            let all_valid_moves = game.movable_pieces_and_possible_moves_of_current_player();
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

pub struct Tree {
    root: Node,
    policy: Option<Box<PolicyValueCallback>> // pick p, move p, value
}

impl Tree {
    pub fn new(policy: Option<Box<PolicyValueCallback>>) -> Self {
        Self { root: Node::new((INVALID_POSITION, INVALID_POSITION), Player::First, 1.), policy }
    }

    pub fn playout(&mut self, game: &Game, ntimes: usize) {
        self.root.player = game.last_player(); // the children of root will be the one play next
        for _ in 0..ntimes {
            self.root.playout_and_update_recursive(game.clone(), self.policy.as_deref());
        }
    }

    pub fn get_action_probs(&self, temp: f32) -> Vec<ActionProb> { // from, to, prob
        debug_assert!(!self.root.children.is_empty());
        let mut visits: Vec<f32> = self.root.children.iter().map(|node| node.n_visits as _).collect();
        for v in visits.iter_mut() {
            *v = libm::logf(*v + 1e-10)
        }
        softmax(&mut visits, temp);
        self.root.children.iter().zip(visits).map(|(node, prob)| (node.action.0, node.action.1, prob)).collect()
    }

    // Choose a subtree and step into next state.
    // This is used in self-play to reuse the searched subtree.
    // note that in evaluation we should create new trees as the subtree is search assuming the opponent uses the same strategy.
    pub fn chroot(&mut self, action: Action) {
        let pos = self.root.children.iter().position(|n| n.action == action).expect("cannot find the action in root children");
        let child = self.root.children.swap_remove(pos);
        self.root = child;
    }

    // sample an action using the root visit counts.
    // exploration_prob: 0 in inference, 0.1 in self-play
    // temperature: 1e-3 in inference, 0.1 in self-play
    pub fn sample_action(&mut self, exploration_prob: f32, temperature: f32) -> Action {
        // let mut children: Vec<_> = self.root.children.iter().map(|c| (c.action, c.n_visits, c.q)).collect();
        // children.sort_by_key(|x| OrderedFloat(-x.2));
        // for i in 0..5 {
        //     if i < children.len() {
        //         println!("{:?}", children[i])
        //     }
        // }

        let acts = self.get_action_probs(temperature);
        // I don't understand why the paper introduces the Dirichlet distribution. It seems to me that it is quivalent to the following implementation.
        let sampled_act = if get_random_float() < exploration_prob {
            uniform_random_choice(&acts)
        } else {
            let sampled_index = sample_categorical(acts.iter().map(|(_, _, p)| *p));
            &acts[sampled_index]
        };
        (sampled_act.0, sampled_act.1)
    }

    pub fn total_visits(&self) -> u32 {
        self.root.n_visits
    }

    pub fn root_value(&self) -> f32 {
        self.root.q
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
}
