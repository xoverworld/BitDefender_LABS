use std::cmp::Ordering;
use std::collections::BinaryHeap;

const ROWS: usize = 51;
const COLS: usize = 90;

#[derive(Copy, Clone, Eq, PartialEq)]
struct State {
    cost: i32,
    position: (usize, usize),
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .cost
            .cmp(&self.cost)
            .then_with(|| self.position.cmp(&other.position))
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Chebyshev distance heuristic, divided by 3 to match the jump size.
fn heuristic(a: (usize, usize), b: (usize, usize)) -> i32 {
    (a.0.abs_diff(b.0).max(a.1.abs_diff(b.1)) / 3) as i32
}

pub fn find_path_macro_grid(
    matrix: &[[i32; COLS]; ROWS],
    start: (usize, usize),
    goal: (usize, usize),
) -> Option<Vec<(usize, usize)>> {
    let mut g_score = [[i32::MAX; COLS]; ROWS];
    g_score[start.0][start.1] = 0;

    let mut came_from = [[None::<(usize, usize)>; COLS]; ROWS];

    let mut open_set = BinaryHeap::new();
    open_set.push(State {
        cost: heuristic(start, goal),
        position: start,
    });

    let directions: [(isize, isize); 8] = [
        (0, 1),
        (1, 0),
        (0, -1),
        (-1, 0),
        (1, 1),
        (-1, -1),
        (1, -1),
        (-1, 1),
    ];

    while let Some(State { cost: _, position }) = open_set.pop() {
        if position == goal {
            // println!("GOOOOOOL");
            let mut path = Vec::new();
            let mut current = goal;

            while current != start {
                path.push(current);
                current = came_from[current.0][current.1].unwrap();
            }
            path.push(start);
            path.reverse();
            return Some(path);
        }

        for (dr, dc) in directions.iter() {
            // Jump directly to the destination tile 3 steps away
            let next_r = position.0 as isize + (dr * 3);
            let next_c = position.1 as isize + (dc * 3);

            if next_r >= 0 && next_r < ROWS as isize && next_c >= 0 && next_c < COLS as isize {
                let r = next_r as usize;
                let c = next_c as usize;

                let terrain_cost = matrix[r][c];

                // Because walls are 3x3, checking the destination tile is enough
                if terrain_cost < 0 {
                    continue;
                }

                // If you want the move to cost "3" instead of "1", change this to:
                // terrain_cost * 3
                let tentative_g = g_score[position.0][position.1].saturating_add(terrain_cost);

                if tentative_g < g_score[r][c] {
                    // println!("{} {}", r, c);

                    came_from[r][c] = Some(position);
                    g_score[r][c] = tentative_g;

                    let f_score = tentative_g + heuristic((r, c), goal);
                    open_set.push(State {
                        cost: f_score,
                        position: (r, c),
                    });
                }
            }
        }
    }

    None
}
