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
        for (dy, dx) in directions.iter() {
            let ny = wall.y + dy;
            let nx = wall.x + dx;
            if ny >= 0 && ny < 90 && nx >= 0 && nx < 51 {
                matrice[ny as usize][nx as usize] = -50;
            }
        }
    }
}

pub fn calculate_score(
    score_matrix: &mut [[i32; 51]; 90],
    path_matrix: &[[i32; 51]; 90],
    enemies: Vec<protocol::Hero>,
    projectiles: Vec<protocol::Projectile>,
) {
    for y in 0..90 {
        for x in 0..51 {
            score_matrix[y][x] = path_matrix[y][x];
        }
    }

    for i in 0..45 {
        for j in 0..51 {
            if path_matrix[0 + i][j] > 0 {
                score_matrix[0 + i][j] += i as i32 / 9;
            }
            if path_matrix[89 - i][j] > 0 {
                score_matrix[89 - i][j] += i as i32 / 9;
            }
        }
    }

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
        let ey = enemy.y as usize;
        let ex = enemy.x as usize;

        score_matrix[ey][ex] -= 10;
        for (dy, dx) in directions {
            let ny = enemy.y + dy;
            let nx = enemy.x + dx;
            if ny >= 0 && ny < 90 && nx >= 0 && nx < 51 {
                score_matrix[ny as usize][nx as usize] -= 8;
            }
        }
        if enemy.cooldown <= 1 {
            for dx in -6..7 {
                for dy in -6..7 {
                    let nx = enemy.x + dx;
                    let ny = enemy.y + dy;
                    if ny >= 0 && ny < 90 && nx >= 0 && nx < 51 {
                        score_matrix[ny as usize][nx as usize] -= 8;
                    }
                }
            }
        } else if enemy.cooldown == 2 {
            for dx in -3..4 {
                for dy in -3..4 {
                    let nx = enemy.x + dx;
                    let ny = enemy.y + dy;
                    if ny >= 0 && ny < 90 && nx >= 0 && nx < 51 {
                        score_matrix[ny as usize][nx as usize] -= 8;
                    }
                }
            }
        }
    }

    for projectile in projectiles {
        let py = projectile.y;
        let px = projectile.x;
        score_matrix[py as usize][px as usize] -= 5;

        let dx = projectile.x - projectile.origin_x;
        let dy = projectile.y - projectile.origin_y;

        if dx.abs() > dy.abs() {
            if dy > 0 {
                // jos
                for i in 0..6 {
                    for j in -6..6 {
                        let ny = py + i;
                        let nx = px + j;
                        if ny >= 0 && ny < 90 && nx >= 0 && nx < 51 {
                            score_matrix[ny as usize][nx as usize] -= 10;
                        }
                    }
                }
            } else {
                // sus
                for i in 0..6 {
                    for j in -6..6 {
                        let ny = py - i;
                        let nx = px + j;
                        if ny >= 0 && ny < 90 && nx >= 0 && nx < 51 {
                            score_matrix[ny as usize][nx as usize] -= 10;
                        }
                    }
                }
            }
        } else {
            if dx > 0 {
                // dreapta
                for i in 0..6 {
                    for j in -6..6 {
                        let ny = py + j;
                        let nx = px + i;
                        if ny >= 0 && ny < 90 && nx >= 0 && nx < 51 {
                            score_matrix[ny as usize][nx as usize] -= 10;
                        }
                    }
                }
            } else {
                // stanga
                for i in 0..6 {
                    for j in -6..6 {
                        let ny = py + j;
                        let nx = px - i;
                        if ny >= 0 && ny < 90 && nx >= 0 && nx < 51 {
                            score_matrix[ny as usize][nx as usize] -= 10;
                        }
                    }
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
    let mut max_score = -100;
    let mut min_distance = i32::MAX;
    let mut final_x = initial_x;
    let mut final_y = initial_y;

    let mut y = 1;
    while y < 90 {
        let mut x = 1;
        while x < 51 {
            let current_score = score_matrix[y][x];
            if current_score >= max_score && path_matrix[y][x] > 0 {
                let distance =
                    (y as i32 - initial_y as i32).abs() + (x as i32 - initial_x as i32).abs();

                if current_score > max_score
                    || (current_score == max_score && distance < min_distance)
                {
                    max_score = current_score;
                    min_distance = distance;
                    final_x = x;
                    final_y = y;
                }
            }
            x += 3;
        }
        y += 3;
    }

    let moves =
        astar::find_path_macro_grid(&path_matrix, (initial_y, initial_x), (final_y, final_x));

    match moves {
        Some(path) => {
            if path.len() > 1 {
                return path[1];
            }
            return path[0];
        }
        None => return (initial_y, initial_x),
    }
}
