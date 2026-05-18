use crate::astar;
use crate::protocol;

pub fn initialize_walls(matrice: &mut [[i32; 51]; 90], walls: &Vec<protocol::Wall>) {
    let directions: [(i32, i32); 8] = [
        (0, 1),
        (1, 0),
        (0, -1),
        (-1, 0),
        (1, 1),
        (-1, -1),
        (1, -1),
        (-1, 1),
    ];
    for wall in walls {
        matrice[wall.y as usize][wall.x as usize] = -50;
        for (i, j) in directions.iter() {
            matrice[(wall.y + i) as usize][(wall.x + j) as usize] = -50;
        }
    }
}

pub fn calculate_score(
    score_matrix: &mut [[i32; 51]; 90],
    path_matrix: &[[i32; 51]; 90],
    enemies: Vec<protocol::Hero>,
    projectiles: Vec<protocol::Projectile>,
) {
    let mut i = 0;
    let mut j;
    while i < 45 {
        j = 0;
        while j < 51 {
            if path_matrix[0 + i][j] > 0 {
                score_matrix[0 + i][j] += (i / 9) as i32;
            }
            if path_matrix[89 - i][j] > 0 {
                score_matrix[89 - i][j] += (i / 9) as i32;
            }
            j += 1;
        }
        i += 1;
    }
    // println!("{:?}", score_matrix);

    let directions: [(i32, i32); 8] = [
        (0, 1),
        (1, 0),
        (0, -1),
        (-1, 0),
        (1, 1),
        (-1, -1),
        (1, -1),
        (-1, 1),
    ];
    for enemy in enemies {
        if enemy.cooldown == 0 {
            score_matrix[enemy.x as usize][enemy.y as usize] -= 10;
            for (x, y) in directions {
                score_matrix[(enemy.x + x) as usize][(enemy.y + y) as usize] -= 8;
            }
        } else {
            score_matrix[enemy.x as usize][enemy.y as usize] += 2;
            for (x, y) in directions {
                score_matrix[(enemy.x + x) as usize][(enemy.y + y) as usize] += 4;
            }
        }
    }
    for projectile in projectiles {
        score_matrix[projectile.x as usize][projectile.y as usize] -= 5;
        if (projectile.x - projectile.origin_x).abs() > (projectile.y - projectile.origin_y).abs() {
            if projectile.y - projectile.origin_y > 0 {
                //jos
                let mut i = 0;
                let mut j;
                while i < 6 {
                    j = -6;
                    while j < 6 {
                        score_matrix[(projectile.x + j) as usize][(projectile.y + i) as usize] -=
                            10;
                        j += 1;
                    }
                    i += 1;
                }
            } else {
                //sus
                let mut i = 0;
                let mut j;
                while i < 6 {
                    j = -6;
                    while j < 6 {
                        score_matrix[(projectile.x + j) as usize][(projectile.y - i) as usize] -=
                            10;
                        j += 1;
                    }
                    i += 1;
                }
            }
        }
        if (projectile.x - projectile.origin_x).abs() < (projectile.y - projectile.origin_y).abs() {
            if projectile.x - projectile.origin_x > 0 {
                //stanga
                let mut i = 0;
                let mut j;
                while i < 6 {
                    j = -6;
                    while j < 6 {
                        score_matrix[(projectile.x - i) as usize][(projectile.y + j) as usize] -=
                            10;
                        j += 1;
                    }
                    i += 1;
                }
            } else {
                //dreapta
                let mut i = 0;
                let mut j;
                while i < 6 {
                    j = -6;
                    while j < 6 {
                        score_matrix[(projectile.x + i) as usize][(projectile.y + j) as usize] -=
                            10;
                        j += 1;
                    }
                    i += 1;
                }
            }
        }
    }
}

pub fn next_move(
    initial_x: usize,
    initial_y: usize,
    path_matrix: &[[i32; 51]; 90],
    score_matrix: &[[i32; 51]; 90],
) -> (usize, usize) {
    let mut max = -100;
    let mut final_x = 0;
    let mut final_y = 0;

    let mut r = 1;
    let mut c = 1;
    while r < 90 {
        c = 1;
        while c < 51 {
            if score_matrix[r][c] > max && path_matrix[r][c] > 0 {
                max = score_matrix[r][c];
                final_x = r;
                final_y = c;
            }
            c += 3;
        }
        r += 3;
    }
    let moves: Option<Vec<(usize, usize)>> =
        astar::find_path_macro_grid(&path_matrix, (initial_x, initial_y), (final_x, final_y));
    // println!("target: {:?}", (final_x, final_y));
    match moves {
        Some(_val) => {
            if _val.len() > 1 {
                return _val[1];
            }
            return _val[0];
        }
        None => return (initial_x, initial_y),
    }
}
