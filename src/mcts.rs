use crate::{Position, game::Game};

fn get_random_number() -> f32 {
    static mut RANDOM: u32 = 39393;

    unsafe {
        RANDOM ^= RANDOM << 13;
        RANDOM ^= RANDOM >> 17;
        RANDOM ^= RANDOM << 5;
        RANDOM as f32 / core::u32::MAX as f32
    }
}

fn uniform_choosing_policy_unnormalized(game: Game) -> Vec<(Position, f32)> {
    game.all_pieces_and_possible_moves_of_current_player().into_iter()
        .filter(|(_, moves)| !moves.is_empty())
        .map(|(pos, _)| (pos, get_random_number()))
        .collect()
}



