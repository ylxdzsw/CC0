use cc0::{self, board, game::{Game, Status}, mcts::Tree};

fn main() {
    let start_time = std::time::Instant::now();
    let mut mcts = Tree::new(None);
    let mut game = Game::new(&board::SMALL_BOARD);
    let mut total_playout = 0;

    loop {
        let n_playout = 20000 - mcts.total_visits();
        mcts.playout(&game, n_playout as _);
        total_playout += n_playout;
        let (from, to) = mcts.sample_action(0., 1e-3);
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
