use crate::error::Result;
use rand::distr::{Bernoulli, Distribution, Uniform};
use rand::Rng;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use serde::Deserialize;
use std::collections::VecDeque;

byond_fn!(fn lavaland_generator_generate(
    width,
    height,
    prefabs_json,
    min_bsp_size,
    max_ratio,
    padding,
    room_fill_percent,
    corridor_width,
    loop_percent,
    noise_percent,
    ca_steps,
    birth_limit,
    survival_limit
) {
    generate_dungeon(
        width, height, prefabs_json, min_bsp_size, max_ratio,
        padding, room_fill_percent, corridor_width, loop_percent,
        noise_percent, ca_steps, birth_limit, survival_limit,
    )
    .ok()
});

// ─── Cell States ───────────────────────────────────────────────────────────────

const DEAD: u8 = 0;       // Dynamic wall
const ALIVE: u8 = 1;      // Dynamic floor
const DEF_ALIVE: u8 = 2;  // Static floor (doesn't change during CA)
const DEF_DEAD: u8 = 3;   // Static wall/indestructible (doesn't change during CA)

// ─── Input Structs ─────────────────────────────────────────────────────────────

fn deserialize_byond_bool<'de, D>(deserializer: D) -> std::result::Result<bool, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    u8::deserialize(deserializer).map(|x| x != 0)
}

#[derive(Deserialize)]
struct PrefabConfig {
    cx: usize,
    cy: usize,
    w: usize,
    h: usize,
    #[serde(default, rename = "isEnclosed", deserialize_with = "deserialize_byond_bool")]
    is_enclosed: bool,
}

// ─── Core Structs ──────────────────────────────────────────────────────────────

struct BSPNode {
    x: usize,
    y: usize,
    w: usize,
    h: usize,
    left: Option<Box<BSPNode>>,
    right: Option<Box<BSPNode>>,
    room: Option<Room>,
}

struct Room {
    x: usize,
    y: usize,
    w: usize,
    h: usize,
    cx: usize,
    cy: usize,
}

#[derive(Clone, Copy)]
struct MSTEdge {
    u: usize,
    v: usize,
    dist: f64,
}


fn generate_dungeon(
    width_str: &str,
    height_str: &str,
    prefabs_json: &str,
    min_bsp_size_str: &str,
    max_ratio_str: &str,
    padding_str: &str,
    room_fill_percent_str: &str,
    corridor_width_str: &str,
    loop_percent_str: &str,
    noise_percent_str: &str,
    ca_steps_str: &str,
    birth_limit_str: &str,
    survival_limit_str: &str,
) -> Result<String> {
    let width = width_str.parse::<usize>()?;
    let height = height_str.parse::<usize>()?;

    let prefabs: Vec<PrefabConfig> = if prefabs_json.is_empty() || prefabs_json == "[]" {
        Vec::new()
    } else {
        serde_json::from_str(prefabs_json)?
    };

    let min_bsp_size = min_bsp_size_str.parse::<usize>().unwrap_or(20);
    let max_ratio    = max_ratio_str.parse::<f64>().unwrap_or(2.5);
    let padding      = padding_str.parse::<usize>().unwrap_or(2);
    let size_scale   = (room_fill_percent_str.parse::<usize>().unwrap_or(80) as f64 / 100.0)
        .clamp(0.0, 1.0);
    let corridor_width  = corridor_width_str.parse::<usize>().unwrap_or(1).max(1);
    let loop_percent    = loop_percent_str.parse::<usize>().unwrap_or(15);
    let noise_percent   = noise_percent_str.parse::<usize>().unwrap_or(48);
    let ca_steps        = ca_steps_str.parse::<usize>().unwrap_or(6);
    let birth_limit     = birth_limit_str.parse::<usize>().unwrap_or(5);
    let survival_limit  = survival_limit_str.parse::<usize>().unwrap_or(4);

    if width == 0 || height == 0 {
        return Ok(String::new());
    }

    let mut rng = rand::rng();

    // Initialize grids
    let mut grid: Vec<Vec<u8>>  = vec![vec![DEAD; height]; width];
    // fixed[x][y] = true → set by prefab/room/corridor; noise must not overwrite it
    let mut fixed: Vec<Vec<bool>> = vec![vec![false; height]; width];

    // Step 1: Apply prefabs first (user-defined locations)
    for prefab in &prefabs {
        apply_prefab(&mut grid, &mut fixed, prefab, width, height);
    }

    // Step 2: BSP partitioning
    let mut root = BSPNode::new(0, 0, width, height);
    root.split(min_bsp_size, max_ratio);
    let mut leaves: Vec<BSPNode> = Vec::new();
    collect_leaves(&root, &mut leaves);
    if leaves.is_empty() {
        leaves.push(BSPNode::new(0, 0, width, height));
    }

    // Step 3: Create rooms in each leaf, using a random scale and dimension
    for leaf in &mut leaves {
        leaf.room = generate_room(leaf, padding, size_scale, &mut rng);
    }

    // Step 4: Adjacency edges + Kruskal MST
    let edges     = build_adjacency_edges(&leaves);
    let mst_edges = kruskal_mst(leaves.len(), &edges, loop_percent, &mut rng);



    // Step 5: Apply rooms to grid as DEF_ALIVE (to prevent them from being eaten by noise), skipping prefab-fixed cells
    leaves.iter().for_each(|leaf| {
        if let Some(ref room) = leaf.room {
            (0..room.w).for_each(|dx| {
                (0..room.h).for_each(|dy| {
                    let gx = room.x + dx;
                    let gy = room.y + dy;
                    if gx < width && gy < height && !fixed[gx][gy] {
                        grid[gx][gy] = DEF_ALIVE;
                        fixed[gx][gy] = true;
                    }
                });
            });
        }
    });

    // Step 6: Carve corridors between rooms along edges, marking them as DEF_ALIVE (this prevents the CA from eating them up later)
    for edge in &mst_edges {
        if let (Some(ra), Some(rb)) = (
            leaves[edge.u].room.as_ref(),
            leaves[edge.v].room.as_ref(),
        ) {
            carve_corridor(
                &mut grid, &mut fixed,
                ra.cx, ra.cy, rb.cx, rb.cy,
                corridor_width, width, height,
                &mut rng,
            );
        }
    }

    // Step 7: Apply noise only to unfixed (empty) cells
    let prob = Bernoulli::new((noise_percent as f64 / 100.0).clamp(0.0, 1.0)).unwrap();
    for x in 0..width {
        for y in 0..height {
            if !fixed[x][y] {
                grid[x][y] = if prob.sample(&mut rng) { ALIVE } else { DEAD };
            }
        }
    }

    // Step 8: CA smoothing
    for _ in 0..ca_steps {
        ca_step(&mut grid, width, height, birth_limit, survival_limit);
    }

    // Step 9: BFS flood-fill island removal from first room center
    if let Some(start) = leaves.iter().find_map(|l| l.room.as_ref().map(|r| (r.cx, r.cy))) {
        flood_fill_island_removal(&mut grid, width, height, start);
    }

    // Output: row-ordered string of all the final tiles (0 = wall, 1 = floor).
    let grid_string: String = (0..height)
        .flat_map(|y| (0..width).map(move |x| (x, y)))
        .map(|(x, y)| match grid[x][y] {
            ALIVE | DEF_ALIVE => '1',
            _ => '0',
        })
        .collect();

    Ok(grid_string)
}


//Apply a prefab to the grid. Marking its tiles as either DEF_DEAD or DEF_ALIVE depending on is_enclosed. This lets us either make the ruins spawn covered in walls, or treated as open (which makes the CA carve it out more)
fn apply_prefab(
    grid: &mut Vec<Vec<u8>>,
    fixed: &mut Vec<Vec<bool>>,
    prefab: &PrefabConfig,
    width: usize,
    height: usize,
) {
    let cx0 = prefab.cx.saturating_sub(1);
    let cy0 = prefab.cy.saturating_sub(1);
    let px = (cx0 as i32 - prefab.w as i32 / 2).max(0) as usize;
    let py = (cy0 as i32 - prefab.h as i32 / 2).max(0) as usize;
    let pw = prefab.w.min(width.saturating_sub(px));
    let ph = prefab.h.min(height.saturating_sub(py));

    for dy in 0..ph {
        for dx in 0..pw {
            let gx = px + dx;
            let gy = py + dy;
            if gx < width && gy < height {
                if prefab.is_enclosed {
                    grid[gx][gy] = DEF_DEAD;
                } else {
                    grid[gx][gy] = DEF_ALIVE;
                }
                fixed[gx][gy] = true;
            }
        }
    }
}

///BSP behavior
impl BSPNode {
    fn new(x: usize, y: usize, w: usize, h: usize) -> Self {
        BSPNode { x, y, w, h, left: None, right: None, room: None }
    }

    fn split(&mut self, min_size: usize, max_ratio: f64) {
        let mut rng = rand::rng();

        let can_split_h = self.h > min_size * 2;
        let can_split_v = self.w > min_size * 2;
        if !can_split_h && !can_split_v {
            return;
        }

        // Random pick which split
        let coin = Bernoulli::new(0.5).unwrap();
        let mut split_horizontal = coin.sample(&mut rng); // true = split by Y
        if self.h > 0 && (self.w as f64 / self.h as f64) >= max_ratio {
            split_horizontal = false; // too wide → split vertically (by X)
        }
        if self.w > 0 && (self.h as f64 / self.w as f64) >= max_ratio {
            split_horizontal = true;  // too tall → split horizontally (by Y)
        }

        // Fall back if forced direction isn't valid
        if split_horizontal && !can_split_h {
            split_horizontal = false;
        } else if !split_horizontal && !can_split_v {
            split_horizontal = true;
        }
        if split_horizontal && !can_split_h { return; }
        if !split_horizontal && !can_split_v { return; }

        if split_horizontal {
            let split_y = Uniform::new(min_size, self.h - min_size).unwrap().sample(&mut rng);
            let mut left  = BSPNode::new(self.x, self.y, self.w, split_y);
            let mut right = BSPNode::new(self.x, self.y + split_y, self.w, self.h - split_y);
            left.split(min_size, max_ratio);
            right.split(min_size, max_ratio);
            self.left  = Some(Box::new(left));
            self.right = Some(Box::new(right));
        } else {
            let split_x = Uniform::new(min_size, self.w - min_size).unwrap().sample(&mut rng);
            let mut left  = BSPNode::new(self.x, self.y, split_x, self.h);
            let mut right = BSPNode::new(self.x + split_x, self.y, self.w - split_x, self.h);
            left.split(min_size, max_ratio);
            right.split(min_size, max_ratio);
            self.left  = Some(Box::new(left));
            self.right = Some(Box::new(right));
        }
    }
}

fn collect_leaves(node: &BSPNode, leaves: &mut Vec<BSPNode>) {
    if node.left.is_none() && node.right.is_none() {
        leaves.push(BSPNode::new(node.x, node.y, node.w, node.h));
        return;
    }
    if let Some(ref left) = node.left {
        collect_leaves(left, leaves);
    }
    if let Some(ref right) = node.right {
        collect_leaves(right, leaves);
    }
}

// Generate a room within a BSP leaf. Scales to a % of the leaf size with a random offset
fn generate_room(leaf: &BSPNode, padding: usize, size_scale: f64, rng: &mut impl Rng) -> Option<Room> {
    let max_w = leaf.w.saturating_sub(padding * 2);
    let max_h = leaf.h.saturating_sub(padding * 2);
    if max_w < 3 || max_h < 3 {
        return None;
    }

    let lo_w = ((max_w as f64 * 0.3) as usize).max(1);
    let hi_w = (max_w as f64 * size_scale) as usize;
    let rw = (if hi_w > lo_w {
        Uniform::new(lo_w, hi_w).unwrap().sample(rng)
    } else {
        lo_w
    })
    .max(3)
    .min(max_w);

    let lo_h = ((max_h as f64 * 0.3) as usize).max(1);
    let hi_h = (max_h as f64 * size_scale) as usize;
    let rh = (if hi_h > lo_h {
        Uniform::new(lo_h, hi_h).unwrap().sample(rng)
    } else {
        lo_h
    })
    .max(3)
    .min(max_h);

    let rx = {
        let lo = padding;
        let hi = leaf.w.saturating_sub(rw + padding);
        let offset = if hi > lo { Uniform::new(lo, hi).unwrap().sample(rng) } else { lo };
        leaf.x + offset
    };
    let ry = {
        let lo = padding;
        let hi = leaf.h.saturating_sub(rh + padding);
        let offset = if hi > lo { Uniform::new(lo, hi).unwrap().sample(rng) } else { lo };
        leaf.y + offset
    };

    Some(Room { x: rx, y: ry, w: rw, h: rh, cx: rx + rw / 2, cy: ry + rh / 2 })
}


// Build edges only between BSP-adjacent leaves that both have rooms
fn build_adjacency_edges(leaves: &[BSPNode]) -> Vec<MSTEdge> {
    let mut edges = Vec::new();
    let n = leaves.len();
    for i in 0..n {
        for j in (i + 1)..n {
            if !rectangles_adjacent(&leaves[i], &leaves[j]) {
                continue;
            }
            let (ra, rb) = match (leaves[i].room.as_ref(), leaves[j].room.as_ref()) {
                (Some(a), Some(b)) => (a, b),
                _ => continue,
            };
            let dist = distance(ra.cx, ra.cy, rb.cx, rb.cy);
            edges.push(MSTEdge { u: i, v: j, dist });
        }
    }
    edges
}

fn rectangles_adjacent(a: &BSPNode, b: &BSPNode) -> bool {
    let a_right  = a.x + a.w;
    let a_bottom = a.y + a.h;
    let b_right  = b.x + b.w;
    let b_bottom = b.y + b.h;
    ((a.x == b_right || b.x == a_right) && !(a.y >= b_bottom || b.y >= a_bottom))
        || ((a.y == b_bottom || b.y == a_bottom) && !(a.x >= b_right || b.x >= a_right))
}

fn distance(x1: usize, y1: usize, x2: usize, y2: usize) -> f64 {
    let dx = x1 as f64 - x2 as f64;
    let dy = y1 as f64 - y2 as f64;
    (dx * dx + dy * dy).sqrt()
}

fn kruskal_mst(n: usize, edges: &[MSTEdge], loop_percent: usize, rng: &mut impl Rng) -> Vec<MSTEdge> {
    let mut sorted = edges.to_vec();
    sorted.sort_by(|a, b| a.dist.partial_cmp(&b.dist).unwrap());

    let mut parent: Vec<usize> = (0..n).collect();
    let mut result = Vec::with_capacity(sorted.len());
    let loop_coin = Bernoulli::new((loop_percent as f64 / 100.0).clamp(0.0, 1.0)).unwrap();

    for edge in &sorted {
        let ru = uf_find(&parent, edge.u);
        let rv = uf_find(&parent, edge.v);
        if ru != rv {
            uf_union(&mut parent, edge.u, edge.v);
            result.push(edge.clone());
        } else if loop_coin.sample(rng) {
            result.push(edge.clone());
        }
    }
    result
}

fn uf_find(parent: &[usize], mut x: usize) -> usize {
    while parent[x] != x {
        x = parent[x];
    }
    x
}

fn uf_union(parent: &mut Vec<usize>, a: usize, b: usize) {
    let ra = uf_find(parent, a);
    let rb = uf_find(parent, b);
    if ra != rb {
        parent[ra] = rb;
    }
}

//Step towards the other edge until we reach it
fn carve_corridor(
    grid: &mut Vec<Vec<u8>>,
    fixed: &mut Vec<Vec<bool>>,
    x1: usize,
    y1: usize,
    x2: usize,
    y2: usize,
    cw: usize,
    width: usize,
    height: usize,
    rng: &mut impl Rng,
) {
    let coin = Bernoulli::new(0.5).unwrap();
    let go_x_first = coin.sample(rng);
    let mut cx = x1 as i32;
    let mut cy = y1 as i32;
    let tx = x2 as i32;
    let ty = y2 as i32;
    let cw_i = cw as i32;

    if go_x_first {
        while cx != tx {
            cx += (tx - cx).signum();
            paint_brush(grid, fixed, cx, cy, cw_i, width, height);
        }
        while cy != ty {
            cy += (ty - cy).signum();
            paint_brush(grid, fixed, cx, cy, cw_i, width, height);
        }
    } else {
        while cy != ty {
            cy += (ty - cy).signum();
            paint_brush(grid, fixed, cx, cy, cw_i, width, height);
        }
        while cx != tx {
            cx += (tx - cx).signum();
            paint_brush(grid, fixed, cx, cy, cw_i, width, height);
        }
    }
}

#[inline]
fn paint_brush(
    grid: &mut Vec<Vec<u8>>,
    fixed: &mut Vec<Vec<bool>>,
    cx: i32,
    cy: i32,
    cw: i32,
    width: usize,
    height: usize,
) {
    for i in 0..cw {
        for j in 0..cw {
            let nx = cx + i;
            let ny = cy + j;
            if nx >= 0 && ny >= 0 {
                let nx = nx as usize;
                let ny = ny as usize;
                if nx < width && ny < height && grid[nx][ny] != DEF_DEAD {
                    grid[nx][ny] = DEF_ALIVE;
                    fixed[nx][ny] = true;
                }
            }
        }
    }
}


// Cellular automata except we use def_alive and def_dead to prevent them from flipping, basically nudging the noise a certain way and preventing it from eating up rooms/corridors/prefabs
fn ca_step(
    grid: &mut Vec<Vec<u8>>,
    width: usize,
    height: usize,
    birth_limit: usize,
    survival_limit: usize,
) {
    let grid_ref: &Vec<Vec<u8>> = grid;
    let new_grid: Vec<Vec<u8>> = (0..width)
        .into_par_iter()
        .map(|x| {
            (0..height)
                .map(|y| {
                    let cell = grid_ref[x][y];
                    if cell == DEF_ALIVE || cell == DEF_DEAD {
                        return cell;
                    }
                    let count = count_alive_neighbors(grid_ref, x, y, width, height);
                    if cell == ALIVE {
                        if count >= survival_limit { ALIVE } else { DEAD }
                    } else {
                        if count >= birth_limit { ALIVE } else { DEAD }
                    }
                })
                .collect()
        })
        .collect();
    *grid = new_grid;
}

fn count_alive_neighbors(grid: &[Vec<u8>], x: usize, y: usize, width: usize, height: usize) -> usize {
    let mut count = 0;
    for dx in -1i32..=1 {
        for dy in -1i32..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                let neighbor = grid[nx as usize][ny as usize];
                if neighbor == ALIVE || neighbor == DEF_ALIVE {
                    count += 1;
                }
            }
        }
    }
    count
}

// kills unreachable ALIVE; DEF_DEAD/DEF_ALIVE is left untouched.
fn flood_fill_island_removal(
    grid: &mut Vec<Vec<u8>>,
    width: usize,
    height: usize,
    start: (usize, usize),
) {
    let (sx, sy) = start;
    if sx >= width || sy >= height {
        return;
    }

    let mut visited = vec![vec![false; height]; width];
    let mut queue: VecDeque<(usize, usize)> = VecDeque::new();
    visited[sx][sy] = true;
    queue.push_back((sx, sy));

    while let Some((cx, cy)) = queue.pop_front() {
        for (ddx, ddy) in [(0i32, 1i32), (0, -1), (1, 0), (-1, 0)] {
            let nx = cx as i32 + ddx;
            let ny = cy as i32 + ddy;
            if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                let nx = nx as usize;
                let ny = ny as usize;
                if !visited[nx][ny] && grid[nx][ny] != DEAD {
                    visited[nx][ny] = true;
                    queue.push_back((nx, ny));
                }
            }
        }
    }

    // For each unvisited pocket: if it contains DEF_ALIVE, keep its ALIVE tiles; otherwise kill them. This preserves natural formations around ruin spawns
    let mut component_visited = vec![vec![false; height]; width];
    for x in 0..width {
        for y in 0..height {
            if visited[x][y] || component_visited[x][y] {
                continue;
            }
            let cell = grid[x][y];
            if cell != ALIVE && cell != DEF_ALIVE {
                continue;
            }
            // BFS the pocket
            let mut alive_cells: Vec<(usize, usize)> = Vec::new();
            let mut has_def_alive = false;
            let mut comp_queue: VecDeque<(usize, usize)> = VecDeque::new();
            component_visited[x][y] = true;
            comp_queue.push_back((x, y));
            while let Some((cx, cy)) = comp_queue.pop_front() {
                match grid[cx][cy] {
                    DEF_ALIVE => has_def_alive = true,
                    ALIVE => alive_cells.push((cx, cy)),
                    _ => {}
                }
                for (ddx, ddy) in [(0i32, 1i32), (0, -1), (1, 0), (-1, 0)] {
                    let nx = cx as i32 + ddx;
                    let ny = cy as i32 + ddy;
                    if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                        let nx = nx as usize;
                        let ny = ny as usize;
                        if !visited[nx][ny] && !component_visited[nx][ny]
                            && (grid[nx][ny] == ALIVE || grid[nx][ny] == DEF_ALIVE)
                        {
                            component_visited[nx][ny] = true;
                            comp_queue.push_back((nx, ny));
                        }
                    }
                }
            }
            if !has_def_alive {
                for (ax, ay) in alive_cells {
                    grid[ax][ay] = DEAD;
                }
            }
        }
    }
}
