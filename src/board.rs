//! The board definition.

use crate::Position;

trait Board {
    fn rank() -> usize;
    fn n_pos() -> usize;
    fn n_pieces() -> usize {
        (1..=Self::rank()).sum()
    }
    fn adj(center: &Position) -> &[Position];
    fn self_base_ids() -> &'static [Position];
    fn oppo_base_ids() -> &'static [Position];
}

/// The standard board has 10 slots in each corner
struct StandardBoard;
impl StandardBoard {
    const ADJ_MATRIX: [[i8; 6]; 121] = [
        [1,2,-1,-1,-1,-1],
        [3,4,2,0,-1,-1],
        [4,5,-1,-1,0,1],
        [6,7,4,1,-1,-1],
        [7,8,5,2,1,3],
        [8,9,-1,-1,2,4],
        [14,15,7,3,-1,-1],
        [15,16,8,4,3,6],
        [16,17,9,5,4,7],
        [17,18,-1,-1,5,8],
        [-1,23,11,-1,-1,-1],
        [23,24,12,-1,-1,10],
        [24,25,13,-1,-1,11],
        [25,26,14,-1,-1,12],
        [26,27,15,6,-1,13],
        [27,28,16,7,6,14],
        [28,29,17,8,7,15],
        [29,30,18,9,8,16],
        [30,31,19,-1,9,17],
        [31,32,20,-1,-1,18],
        [32,33,21,-1,-1,19],
        [33,34,22,-1,-1,20],
        [34,-1,-1,-1,-1,21],
        [-1,35,24,11,10,-1],
        [35,36,25,12,11,23],
        [36,37,26,13,12,24],
        [37,38,27,14,13,25],
        [38,39,28,15,14,26],
        [39,40,29,16,15,27],
        [40,41,30,17,16,28],
        [41,42,31,18,17,29],
        [42,43,32,19,18,30],
        [43,44,33,20,19,31],
        [44,45,34,21,20,32],
        [45,-1,-1,22,21,33],
        [-1,46,36,24,23,-1],
        [46,47,37,25,24,35],
        [47,48,38,26,25,36],
        [48,49,39,27,26,37],
        [49,50,40,28,27,38],
        [50,51,41,29,28,39],
        [51,52,42,30,29,40],
        [52,53,43,31,30,41],
        [53,54,44,32,31,42],
        [54,55,45,33,32,43],
        [55,-1,-1,34,33,44],
        [-1,56,47,36,35,-1],
        [56,57,48,37,36,46],
        [57,58,49,38,37,47],
        [58,59,50,39,38,48],
        [59,60,51,40,39,49],
        [60,61,52,41,40,50],
        [61,62,53,42,41,51],
        [62,63,54,43,42,52],
        [63,64,55,44,43,53],
        [64,-1,-1,45,44,54],
        [65,66,57,47,46,-1],
        [66,67,58,48,47,56],
        [67,68,59,49,48,57],
        [68,69,60,50,49,58],
        [69,70,61,51,50,59],
        [70,71,62,52,51,60],
        [71,72,63,53,52,61],
        [72,73,64,54,53,62],
        [73,74,-1,55,54,63],
        [75,76,66,56,-1,-1],
        [76,77,67,57,56,65],
        [77,78,68,58,57,66],
        [78,79,69,59,58,67],
        [79,80,70,60,59,68],
        [80,81,71,61,60,69],
        [81,82,72,62,61,70],
        [82,83,73,63,62,71],
        [83,84,74,64,63,72],
        [84,85,-1,-1,64,73],
        [86,87,76,65,-1,-1],
        [87,88,77,66,65,75],
        [88,89,78,67,66,76],
        [89,90,79,68,67,77],
        [90,91,80,69,68,78],
        [91,92,81,70,69,79],
        [92,93,82,71,70,80],
        [93,94,83,72,71,81],
        [94,95,84,73,72,82],
        [95,96,85,74,73,83],
        [96,97,-1,-1,74,84],
        [98,99,87,75,-1,-1],
        [99,100,88,76,75,86],
        [100,101,89,77,76,87],
        [101,102,90,78,77,88],
        [102,103,91,79,78,89],
        [103,104,92,80,79,90],
        [104,105,93,81,80,91],
        [105,106,94,82,81,92],
        [106,107,95,83,82,93],
        [107,108,96,84,83,94],
        [108,109,97,85,84,95],
        [109,110,-1,-1,85,96],
        [-1,-1,99,86,-1,-1],
        [-1,-1,100,87,86,98],
        [-1,-1,101,88,87,99],
        [-1,-1,102,89,88,100],
        [-1,111,103,90,89,101],
        [111,112,104,91,90,102],
        [112,113,105,92,91,103],
        [113,114,106,93,92,104],
        [114,-1,107,94,93,105],
        [-1,-1,108,95,94,106],
        [-1,-1,109,96,95,107],
        [-1,-1,110,97,96,108],
        [-1,-1,-1,-1,97,109],
        [-1,115,112,103,102,-1],
        [115,116,113,104,103,111],
        [116,117,114,105,104,112],
        [117,-1,-1,106,105,113],
        [-1,118,116,112,111,-1],
        [118,119,117,113,112,115],
        [119,-1,-1,114,113,116],
        [-1,120,119,116,115,-1],
        [120,-1,-1,117,116,118],
        [-1,-1,-1,119,118,-1]
    ];
    const SELF_BASE_IDS: [Position; 10] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
    const OPPO_BASE_IDS: [Position; 10] = [111, 112, 113, 114, 115, 116, 117, 118, 119, 120];
}

impl Board for StandardBoard {
    fn rank() -> usize { 4 }
    fn n_pos() -> usize { 121 }
    fn adj(center: &Position) -> &[Position] {
        &Self::ADJ_MATRIX[*center as usize]
    }
    fn self_base_ids() -> &'static [Position] {
        &Self::SELF_BASE_IDS
    }
    fn oppo_base_ids() -> &'static [Position] {
        &Self::OPPO_BASE_IDS
    }
}

/// The small board has only 6 slots in each corner
struct SmallBoard;
impl SmallBoard {
    const ADJ_MATRIX: [[i8; 6]; 73] = [
        [1,2,-1,-1,-1,-1],
        [3,4,2,0,-1,-1],
        [4,5,-1,-1,0,1],
        [9,10,4,1,-1,-1],
        [10,11,5,2,1,3],
        [11,12,-1,-1,2,4],
        [-1,16,7,-1,-1,-1],
        [16,17,8,-1,-1,6],
        [17,18,9,-1,-1,7],
        [18,19,10,3,-1,8],
        [19,20,11,4,3,9],
        [20,21,12,5,4,10],
        [21,22,13,-1,5,11],
        [22,23,14,-1,-1,12],
        [23,24,15,-1,-1,13],
        [24,-1,-1,-1,-1,14],
        [-1,25,17,7,6,-1],
        [25,26,18,8,7,16],
        [26,27,19,9,8,17],
        [27,28,20,10,9,18],
        [28,29,21,11,10,19],
        [29,30,22,12,11,20],
        [30,31,23,13,12,21],
        [31,32,24,14,13,22],
        [32,-1,-1,15,14,23],
        [-1,33,26,17,16,-1],
        [33,34,27,18,17,25],
        [34,35,28,19,18,26],
        [35,36,29,20,19,27],
        [36,37,30,21,20,28],
        [37,38,31,22,21,29],
        [38,39,32,23,22,30],
        [39,-1,-1,24,23,31],
        [40,41,34,26,25,-1],
        [41,42,35,27,26,33],
        [42,43,36,28,27,34],
        [43,44,37,29,28,35],
        [44,45,38,30,29,36],
        [45,46,39,31,30,37],
        [46,47,-1,32,31,38],
        [48,49,41,33,-1,-1],
        [49,50,42,34,33,40],
        [50,51,43,35,34,41],
        [51,52,44,36,35,42],
        [52,53,45,37,36,43],
        [53,54,46,38,37,44],
        [54,55,47,39,38,45],
        [55,56,-1,-1,39,46],
        [57,58,49,40,-1,-1],
        [58,59,50,41,40,48],
        [59,60,51,42,41,49],
        [60,61,52,43,42,50],
        [61,62,53,44,43,51],
        [62,63,54,45,44,52],
        [63,64,55,46,45,53],
        [64,65,56,47,46,54],
        [65,66,-1,-1,47,55],
        [-1,-1,58,48,-1,-1],
        [-1,-1,59,49,48,57],
        [-1,-1,60,50,49,58],
        [-1,67,61,51,50,59],
        [67,68,62,52,51,60],
        [68,69,63,53,52,61],
        [69,-1,64,54,53,62],
        [-1,-1,65,55,54,63],
        [-1,-1,66,56,55,64],
        [-1,-1,-1,-1,56,65],
        [-1,70,68,61,60,-1],
        [70,71,69,62,61,67],
        [71,-1,-1,63,62,68],
        [-1,72,71,68,67,-1],
        [72,-1,-1,69,68,70],
        [-1,-1,-1,71,70,-1]
    ];
    const SELF_BASE_IDS: [Position; 6] = [0, 1, 2, 3, 4, 5];
    const OPPO_BASE_IDS: [Position; 6] = [67, 68, 69, 70, 71, 72];
}

impl Board for SmallBoard {
    fn rank() -> usize { 3 }
    fn n_pos() -> usize { 73 }
    fn adj(center: &Position) -> &[Position] {
        &Self::ADJ_MATRIX[*center as usize]
    }
    fn self_base_ids() -> &'static [Position] {
        &Self::SELF_BASE_IDS
    }
    fn oppo_base_ids() -> &'static [Position] {
        &Self::OPPO_BASE_IDS
    }
}
