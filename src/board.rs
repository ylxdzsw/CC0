//! The board definition of a chinease checker game.

use crate::Position;

//
//  2 3
// 1 0 4
//  6 5
//

trait Board {
    fn adj(center: &Position) -> &[Position];
}

/// The standard board has 10 slots in each corner
struct StandardBoard;
impl StandardBoard {
    const ADJ_MATRIX: [[u8; 0]; 1] = [[]];
}

impl Board for StandardBoard {

    fn adj(center: &Position) -> &[Position] {
        return &Self::ADJ_MATRIX[*center as usize]
    }
}

/// The small board has only 6 slots in each corner
struct SmallBoard {

}
