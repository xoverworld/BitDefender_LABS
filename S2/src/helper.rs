use crate::astar;
use crate::protocol;

pub fn initializeWalls(matrice: &mut [[i32; 90]; 51], walls: &Vec<protocol::Wall>) {
    for wall in walls {
        matrice[wall.x as usize][wall.y as usize] = -5;
    }
}

pub fn nextMove(
    initial_x: usize,
    initial_y: usize,
    path_matrix: &[[i32; 90]; 51],
    score_matrix: &[[i32; 90]; 51],
) -> (usize, usize) {
    let mut max = 0;
    let mut final_x = 0;
    let mut final_y = 0;
    for r in 0..51 {
        for c in 0..90 {
            if score_matrix[r][c] > max {
                max = score_matrix[r][c];
                final_x = r;
                final_y = c;
            }
        }
    }
    let moves: Option<Vec<(usize, usize)>> =
        astar::find_path_macro_grid(&path_matrix, (initial_x, initial_y), (final_x, final_y));
    // println!("{:?}", moves);
    return moves.unwrap()[1];
}
