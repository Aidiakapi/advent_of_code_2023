use bitvec::prelude::*;
framework::day!(22, parse => pt1, pt2);

type Vec2 = framework::vecs::Vec2<u32>;
type Vec3 = framework::vecs::Vec3<u32>;
type Vec2us = framework::vecs::Vec2<usize>;

#[derive(Debug, Clone, Default)]
struct Cell {
    height: u32,
    brick_index: Option<u32>,
}

#[derive(Debug, Clone, Default)]
struct SupportInfo {
    supporting: ArrayVec<u32, 8>,
    supported_by: u32,
}

#[derive(Debug, Clone)]
struct BrickStructure {
    bricks: Vec<(Vec3, Vec3)>,
    heightmap: VecGrid<Cell>,
    support_set: BitVec<u64, LocalBits>,
    support_structure: Vec<SupportInfo>,
}

impl BrickStructure {
    fn new(bricks: &[(Vec3, Vec3)]) -> Result<BrickStructure> {
        let mut bricks = bricks.to_vec();
        let (mut min_xy, mut max_xy) = (Vec2::from(u32::MAX), Vec2::from(u32::MIN));
        for brick in &mut bricks {
            *brick = brick.0.minmax_comp(brick.1);
            min_xy = min_xy.min_comp(brick.0.xy());
            max_xy = max_xy.max_comp(brick.1.xy());
        }
        bricks.sort_unstable_by(|a, b| a.0.z.cmp(&b.0.z));

        if min_xy != Vec2::zero() {
            return Err(Error::InvalidInput("expected bricks at zero"));
        }

        let heightmap = VecGrid::new((max_xy + 1).to_usize(), |_| Cell::default());

        let support_set = BitVec::repeat(false, bricks.len());
        let mut support_structure = Vec::new();
        support_structure.resize(bricks.len(), SupportInfo::default());

        Ok(Self {
            bricks,
            heightmap,
            support_set,
            support_structure,
        })
    }

    fn fall(&mut self) -> u32 {
        let mut fall_count = 0;
        for (index, brick) in self.bricks.iter_mut().enumerate() {
            let supported_at = iter_each_xy(*brick).map(|xy| self.heightmap[xy].height);
            let supported_at = supported_at.max().unwrap();

            self.support_set.fill(false);
            iter_each_xy(*brick)
                .filter_map(|xy| {
                    let cell = &self.heightmap[xy];
                    cell.brick_index.filter(|_| cell.height == supported_at)
                })
                .for_each(|brick_index| self.support_set.set(brick_index as usize, true));
            let mut supported_by_count = 0;
            for supported_by_index in self.support_set.iter_ones() {
                (self.support_structure[supported_by_index].supporting).push(index as u32);
                supported_by_count += 1;
            }
            self.support_structure[index].supported_by = supported_by_count;

            let dz = brick.1.z - brick.0.z;
            if brick.0.z != supported_at + 1 {
                fall_count += 1;
            }
            brick.0.z = supported_at + 1;
            brick.1.z = supported_at + 1 + dz;

            let cell = Cell {
                height: supported_at + 1 + dz,
                brick_index: Some(index as u32),
            };
            for xy in iter_each_xy(*brick) {
                self.heightmap[xy] = cell.clone();
            }
        }
        fall_count
    }
}

fn iter_each_xy(brick: (Vec3, Vec3)) -> impl Iterator<Item = Vec2us> {
    let (min, max) = (brick.0.xy(), brick.1.xy() + 1);
    (min.x..max.x).flat_map(move |x| (min.y..max.y).map(move |y| Vec2::new(x, y).to_usize()))
}

fn pt1(bricks: &[(Vec3, Vec3)]) -> Result<u32> {
    let mut bricks = BrickStructure::new(bricks)?;
    bricks.fall();

    Ok((bricks.support_structure.iter())
        .filter(|structure| {
            (structure.supporting.iter())
                .all(|&index| bricks.support_structure[index as usize].supported_by > 1)
        })
        .count() as u32)
}

fn pt2(bricks: &[(Vec3, Vec3)]) -> Result<u32> {
    let mut bricks = BrickStructure::new(bricks)?;
    bricks.fall();

    let bricks_to_disintegrate = bricks.support_structure.iter().positions(|structure| {
        !(structure.supporting.iter())
            .all(|&index| bricks.support_structure[index as usize].supported_by > 1)
    });

    let mut total_fall_count = 0;
    for index in bricks_to_disintegrate {
        let mut new_bricks = bricks.clone();
        new_bricks.bricks.remove(index);
        new_bricks.heightmap.cells_mut().fill(Cell::default());
        new_bricks.support_structure.fill(SupportInfo::default());

        total_fall_count += new_bricks.fall();
    }

    Ok(total_fall_count)
}

fn parse(input: &[u8]) -> Result<Vec<(Vec3, Vec3)>> {
    use parsers::*;
    let nr = number::<u32>();
    let comma_nr = token(b',').then(nr);
    let position = (nr.and(comma_nr).and(comma_nr)).map(|((x, y), z)| Vec3::new(x, y, z));
    let brick = position.and(token(b'~').then(position));
    let brick = brick.map_res(|(a, b)| {
        let comps = a.eq_comp(b);
        let sum = u8::from(comps.x) + u8::from(comps.y) + u8::from(comps.z);
        if sum >= 2 {
            Ok((a, b))
        } else {
            Err(ParseError::Custom(
                "bricks may not differ in more than one component",
            ))
        }
    });
    brick.sep_by(token(b'\n')).execute(input)
}

tests! {
    const EXAMPLE: &'static [u8] = b"\
1,0,1~1,2,1
0,0,2~2,0,2
0,2,3~2,2,3
0,0,4~0,2,4
2,0,5~2,2,5
0,1,6~2,1,6
1,1,8~1,1,9
";

    test_pt!(parse, pt1, EXAMPLE => 5);
    test_pt!(parse, pt2, EXAMPLE => 7);
}
