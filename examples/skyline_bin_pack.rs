//! My implementation about Skyline bin pack algorithm
//! Based on : "A Thousand Ways to Pack the Bin - A Practical Approach to Two-Dimensional Rectangle Bin Packing."

#[derive(Debug)]
struct IRect {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

/// Represents a single level line
struct SkylineNode {
    /// The starting x-coordinate
    x: u32,
    /// The y-coordinate of the skyline level line
    y: u32,
    /// The line width. The ending coordinate is (x + width - 1)
    width: u32,
}

struct SkylineBinPack {
    width: u32,
    height: u32,
    sky_line: Vec<SkylineNode>,
}

impl SkylineBinPack {
    fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            sky_line: vec![SkylineNode {
                x: 0,
                y: 0,
                width: width,
            }],
        }
    }

    fn insert(&mut self, width: u32, height: u32) -> Option<IRect> {
        let (index, node, _best_width, _best_height) = self.find_position(width, height);

        if index.is_none() {
            return None;
        }

        let index = index.unwrap();

        self.add_skyline_level(index, &node);

        return Some(node);
    }

    fn find_position(&self, width: u32, height: u32) -> (Option<u32>, IRect, u32, u32) {
        let mut best_height = u32::MAX;
        let mut best_width = u32::MAX;
        let mut best_index: Option<u32> = None;

        let mut new_node = IRect {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        };

        for (i, line) in self.sky_line.iter().enumerate() {
            let (fit, y) = self.rectangle_fits(i, width, height);

            if !fit {
                continue;
            }

            if y + height < best_height || (y + height == best_height && line.width < best_width) {
                best_height = y + height;
                best_width = line.width;
                best_index = Some(i as u32);

                new_node.x = line.x;
                new_node.y = y;
                new_node.width = width;
                new_node.height = height;
            }
        }

        return (best_index, new_node, best_width, best_height);
    }

    fn rectangle_fits(&self, index: usize, width: u32, height: u32) -> (bool, u32) {
        let x = self.sky_line[index].x;
        if x + width > self.width {
            return (false, 0);
        }

        let mut width_left = width as i64;

        let mut i = index;

        let mut y = self.sky_line[index].y;

        while width_left > 0 {
            y = y.max(self.sky_line[i].y);

            if y + height > self.height {
                return (false, 0);
            }

            width_left -= self.sky_line[i].width as i64;

            i = i + 1;
        }

        return (true, y);
    }

    fn add_skyline_level(&mut self, index: u32, rect: &IRect) {
        let new_node = SkylineNode {
            x: rect.x,
            y: rect.y + rect.height,
            width: rect.width,
        };

        self.sky_line.insert(index as usize, new_node);

        let mut i = index as usize + 1;

        while i < self.sky_line.len() {
            if self.sky_line[i - 1].x < self.sky_line[i].x + self.sky_line[i - 1].width {
                let shrink =
                    self.sky_line[i - 1].x + self.sky_line[i - 1].width - self.sky_line[i].x;

                self.sky_line[i].x += shrink;
                let left = self.sky_line[i].width as i64 - shrink as i64;

                if left <= 0 {
                    self.sky_line.remove(i);
                    i = i - 1;
                } else {
                    self.sky_line[i].width = left as u32;
                    break;
                }
            } else {
                break;
            }

            i = i + 1;
        }

        self.merge_skyline();
    }

    fn merge_skyline(&mut self) {
        let mut i = 0;

        while i < self.sky_line.len() - 1 {
            if self.sky_line[i].y == self.sky_line[i + 1].y {
                self.sky_line[i].width += self.sky_line[i + 1].width;
                self.sky_line.remove(i + 1);
            }

            i = i + 1;
        }
    }
}

fn main() {
    let mut sky_line_pack = SkylineBinPack::new(4096, 4096);

    let rect1 = sky_line_pack.insert(32, 32);

    println!("rect1 is {:?}", rect1);

    let rect2 = sky_line_pack.insert(32, 32);

    println!("rect2 is {:?}", rect2);

    println!("rect3 is {:?}", sky_line_pack.insert(40, 40));

    println!("rect4 is {:?}", sky_line_pack.insert(20, 20));

    println!("rect6 is {:?}", sky_line_pack.insert(50, 50));
}
