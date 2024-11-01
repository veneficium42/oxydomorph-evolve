use bevy::math::{IVec3, Vec3};
use rand::Rng;

/*
genome = dict ([(i,gene['lv']) for i in range(1,gene['num']+1)])
genome[9] = gene['ln']
*/
const fn genome() -> [(i32, i32); 9] {
    let mut genome = [(-9, 9); 9];
    genome[8] = (3, 9);
    genome
}

const MAX_SEGMENTS: usize = 1024;

/// Configuration passed down from the GUI
pub struct Config {
    pub columns: usize,
    pub rows: usize,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            columns: 3,
            rows: 3,
        }
    }
}
impl Config {
    pub fn default() -> Self {
        Default::default()
    }
}

/// Definition of conditions for all the biomorphs and groups them together
/// the Matrix is the grid displayed
#[derive(Default)]
pub struct Matrix {
    pub biomorphs: Vec<Biomorph>,
    dx: [i32; 9],
    dy: [i32; 9],
}

impl Matrix {
    /// Runs when Matrix first initialized
    pub fn initial_setup(config: &Config) -> Self {
        // initialize vector on self
        let mut biomorphs = Vec::with_capacity(config.columns * config.rows);
        // loop thru every cell in the matrix
        for _i in 0..(config.columns * config.rows) {
            biomorphs.push(Biomorph::initial());
        }

        let mut dx: [i32; 9] = [0; 9];
        for i in 0..8 {
            dx[i] = i as i32;
        }

        let mut dy: [i32; 9] = [0; 9];
        for i in 0..8 {
            dy[i] = i as i32;
        }

        Matrix { biomorphs, dx, dy }
    }

    pub fn develop(&mut self, biomorph_index: usize) {
        self.biomorphs[biomorph_index].develop(0, 0, 2, self.dx, self.dy);
    }

    pub fn reproduce(&mut self, biomorph_index: usize) {
        let biomorph = self.biomorphs[biomorph_index].clone();
        for i in 0..self.biomorphs.len() {
            self.biomorphs[i] = biomorph.reproduce();
            self.biomorphs[i].develop(0, 0, 2, self.dx, self.dy);
        }
    }
}

/// Definition for the data that a biomorph has
#[derive(Clone)]
pub struct Biomorph {
    pub genes: [i32; 9],
    pub segment_list: Vec<Segment>,
}

impl Biomorph {
    fn initial() -> Biomorph {
        let genome = genome();
        let mut genes = [0; 9];
        for i in 0..8 {
            genes[i] = rand::thread_rng().gen_range(genome[i].0..(genome[i].1 + 1));
        }

        genes[8] = rand::thread_rng().gen_range((genome[7].1 - 3)..(genome[7].1 + 1));

        let segment_list: Vec<Segment> = Vec::with_capacity(MAX_SEGMENTS);

        Biomorph {
            genes,
            segment_list,
        }
    }

    fn reproduce(&self) -> Biomorph {
        let i = rand::thread_rng().gen_range(0..9);
        let genome = genome();
        let mut genes = self.genes;

        genes[i] += if rand::thread_rng().gen_bool(0.5) {
            1
        } else {
            -1
        };

        if genes[i] < genome[i].0 {
            genes[i] = genome[i].0 + 1;
        } else if genes[i] > genome[i].1 {
            genes[i] = genome[i].1 - 1;
        }

        let segment_list: Vec<Segment> = Vec::with_capacity(MAX_SEGMENTS);

        Biomorph {
            genes,
            segment_list,
        }
    }

    //TODO: figure out what the fuck this function is doing
    // probably mixing around the genes?? maybe??
    fn plugin(genes: [i32; 9], mut dx: [i32; 9], mut dy: [i32; 9]) -> (i32, [i32; 9], [i32; 9]) {
        let order = genes[8];
        dx[3] = genes[1];
        dx[4] = genes[2];
        dx[5] = genes[3];
        dx[1] = -dx[3];
        dx[0] = -dx[4];
        dx[2] = 0;
        dx[6] = 0;
        dx[7] = -dx[5];
        dy[2] = genes[4];
        dy[3] = genes[5];
        dy[4] = genes[6];
        dy[5] = genes[7];
        dy[6] = genes[8];
        dy[0] = dy[4];
        dy[1] = dy[3];
        dy[7] = dy[5];
        (order, dx, dy)
    }

    //TODO: remove recursion cuz recursion sucks tbh
    fn tree(&mut self, x: i32, y: i32, length: i32, dir: i32, dx: [i32; 9], dy: [i32; 9]) {
        let _dir = (dir.rem_euclid(8)) as usize;
        let new_x = x + length * dx[_dir];
        let new_y = y + length * dy[_dir];

        self.segment_list.push(Segment {
            start: IVec3 { x, y, z: 0 },
            end: IVec3 {
                x: new_x,
                y: new_y,
                z: 0,
            },
        });

        if length > 0 {
            self.tree(new_x, new_y, length - 1, dir - 1, dx, dy);
            self.tree(new_x, new_y, length - 1, dir + 1, dx, dy);
        }
    }

    fn develop(&mut self, start_x: i32, start_y: i32, start_dir: i32, dx: [i32; 9], dy: [i32; 9]) {
        if self.segment_list.len() > 1 {
            self.segment_list = Vec::with_capacity(MAX_SEGMENTS);
        }

        let (order, new_dx, new_dy) = Biomorph::plugin(self.genes, dx, dy);
        self.tree(start_x, start_y, order, start_dir, new_dx, new_dy);
    }

    pub fn bounding_box(&self) -> (Vec3, Vec3) {
        let mut min = Vec3::MAX;
        let mut max = Vec3::MIN;
        for segment in &self.segment_list {
            min = min.min(segment.start.as_vec3());
            min = min.min(segment.end.as_vec3());

            max = max.max(segment.start.as_vec3());
            max = max.max(segment.end.as_vec3());
        }
        return (min, max);
    }

    pub fn center(&self) -> Vec3 {
        let (min, max) = self.bounding_box();
        let center = (max + min) / 2.0;
        center
    }
}

/// Definition of a segment, every biomorph has a set of segments and they are rendered one by one
#[derive(Clone)]
pub struct Segment {
    pub start: IVec3,
    pub end: IVec3,
}

impl Segment {
    fn initial() -> Segment {
        Segment {
            start: IVec3::ZERO,
            end: IVec3::ZERO,
        }
    }
}
