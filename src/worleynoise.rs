use crate::error::Result;
use rand::*;
use std::fmt::Write;
use std::thread;

byond_fn! { worley_generate(region_size, threshold, width, height) {
    worley_noise(region_size, threshold, width, height).ok()
} }

// This is a quite complex algorithm basically what it does is it creates 2 maps, one filled with cells and the other with 'regions' that map onto these cells.
// Each region can spawn 1 node, the cell then determines wether it is true or false depending on the distance from it to the nearest node in the region minus the second closest node.
// If this distance is greater than the threshold then the cell is true, otherwise it is false.
fn worley_noise(str_reg_size : &str, str_positive_threshold: &str, str_width: &str, str_height: &str) -> Result<String>{

    let region_size = str_reg_size.parse::<i32>()?;
    let positive_threshold = str_positive_threshold.parse::<f32>()?;
    let width = str_width.parse::<i32>()?;
    let height = str_height.parse::<i32>()?;

    //i fucking mixed up width and height again. it really doesnt matter but here is a comment just warning you.
    let mut map = Map::new(region_size,height,width);

    map.generate_noise(positive_threshold as f32);

    let mut output = String::new();

    for row in map.cell_map{
        for cell in row{
            if cell.value {
                output.append_str("1");
            } else {
                output.append_str("0");
            }
        }
    }
    Ok(output)
}


struct Map{
    region_size : i32,
    region_map : Vec<Vec<Rc<Region>>>,
    cell_map : Vec<Vec<Cell>>,
    cell_map_width : i32,
    cell_map_height : i32,
}

impl Map{
    fn new(region_size : i32,  cell_map_width : i32, cell_map_height : i32) -> Map{
        let mut map = Map{
            region_size : region_size,
            region_map : Vec::new(),
            cell_map : Vec::new(),
            cell_map_width : cell_map_width,
            cell_map_height : cell_map_height,
        };

        map.init_regions();

        for x in 0..cell_map_width{
            map.cell_map.push(Vec::new());
            for y in 0..cell_map_height{
                let cell = Cell::new(x,y,map.region_map[(x / region_size) as usize][(y / region_size) as usize].clone());
                        map.cell_map[(x) as usize].push(cell);
            }
        }
        map
    }
    fn init_regions(&mut self){
        let mut rng = rand::thread_rng();

        let regions_x = self.cell_map_width / self.region_size;
        let regions_y = self.cell_map_height / self.region_size;

        for i in 0..regions_x {
            self.region_map.push(Vec::new());
            for j in 0..regions_y {
                let mut region =Region::new(i,j);
                let xcord = rng.gen_range(0..self.region_size);
                let ycord = rng.gen_range(0..self.region_size);
                let node = Node::new(xcord + i*self.region_size ,ycord + j*self.region_size );
                region.node = Some(node);

                let  rcregion = Rc::new(region);

                self.region_map[i as usize].push(rcregion);
            }
        }
    }

    fn get_regions_in_bound(&self, x : i32, y : i32, radius : i32) -> Vec<&Region>{
        let mut regions = Vec::new();
        let x_min = x - radius;
        let x_max = x + radius;
        let y_min = y - radius;
        let y_max = y + radius;
        for i in x_min..x_max {
            for j in y_min..y_max {

                let region_x = i;
                let region_y = j;
                if region_x != x && region_y != y {


                    if region_x >= 0 && region_x < self.region_map.len() as i32 && region_y >= 0 && region_y < self.region_map[region_x as usize].len() as i32 {
                        let region = &self.region_map[region_x as usize][region_y as usize];
                        regions.push(region.as_ref());
                    }
                }else{
                    continue;
                }
            }
        }
        regions
    }

    fn generate_noise(&mut self,threshold : f32){
        for i in 0..self.cell_map.len() {
            for j in 0..self.cell_map[i as usize].len() {
                let cell =  &self.cell_map[i as usize][j as usize];
                let region = &self.region_map[cell.region.as_ref().reg_x as usize][cell.region.as_ref().reg_y as usize];
                let neighbours = self.get_regions_in_bound(region.reg_x,region.reg_y,3);


                let mut node_vec = Vec::new();
                node_vec.push(region.as_ref().node.as_ref().unwrap());
                for neighbour in neighbours {
                    let node = neighbour.node.as_ref().unwrap();
                    node_vec.push(node);

                }

                dmsort::sort_by(&mut node_vec, |a,b| quick_distance_from_to(cell.x, cell.y, a.x , a.y).partial_cmp(&quick_distance_from_to(cell.x, cell.y, b.x , b.y)).unwrap());
                let dist = distance_from_to(cell.x, cell.y, node_vec[0].x , node_vec[0].y) - distance_from_to(cell.x, cell.y, node_vec[1].x , node_vec[1].y);
                let mutable_cell = &mut self.cell_map[i as usize][j as usize];
                if dist.abs() > threshold {
                    mutable_cell.value = true;
                }
            }
        }
    }
}

fn distance_from_to(x1 : i32, y1 : i32, x2 : i32, y2 : i32) -> f32{
    let x_diff = x1 - x2;
    let y_diff = y1 - y2;
    let distance = (((x_diff * x_diff) + (y_diff * y_diff)) as f32).sqrt();
    distance
}

fn quick_distance_from_to(x1 : i32, y1 : i32, x2 : i32, y2 : i32) -> f32{
    let x_diff = x1 - x2;
    let y_diff = y1 - y2;
    let distance = (x_diff.abs() + y_diff.abs()) as f32;
    distance
}

struct Cell {
    x: i32,
    y: i32,
    value: bool,
    region : Rc<Region>
}

impl Cell {
    fn new(x: i32, y: i32, region: Rc<Region>) -> Cell {
        Cell {
            x: x,
            y: y,
            value: false,
            region: region,
        }
    }
}

struct Region{
    reg_x: i32,
    reg_y: i32,
    node: Option<Node>,
}

impl Region{
    fn new(reg_x: i32, reg_y: i32) -> Region{
        Region{
            reg_x: reg_x,
            reg_y: reg_y,
            node: None
        }
    }
}

struct Node {
    x: i32,
    y: i32,
}

impl Node {
    fn new(x: i32, y: i32) -> Node {
        Node {
            x: x,
            y: y,
        }
    }
}

