use std::iter::once;

use super::Gpt;
use crate::region::Region;

impl Gpt {
    pub fn find_free_regions(&self) -> Vec<Region<u64>> {
        let usable_region = Region::new(self.first_usable_lba, self.last_usable_lba);
        let mut it = self
            .partitions
            .iter()
            .filter_map(|x| x.as_ref().map(|x| Region::new(x.start_lba, x.end_lba)));

        find_free_regions(usable_region, &mut it)
    }
}

fn find_free_regions(
    usable_region: Region<u64>,
    region_it: &mut dyn Iterator<Item = Region<u64>>,
) -> Vec<Region<u64>> {
    let mut usable_regions = once(Some(usable_region)).collect::<Vec<_>>();

    for used_region in region_it {
        for i in 0..usable_regions.len() {
            if let Some(usable) = &usable_regions[i] {
                let (first, second) = usable.substract(&used_region);
                if let Some(first) = first {
                    usable_regions[i] = Some(first);

                    if let Some(second) = second {
                        usable_regions.push(Some(second));
                    }
                } else {
                    usable_regions[i] = None;
                }
            }
        }
    }

    usable_regions.iter().copied().flatten().collect()
}

#[cfg(test)]
#[test]
fn test_find_free_regions() {
    let usable_region = Region::new(34, 2097116);

    macro_rules! test {
        ($({$x:expr, $y:expr}),+ expect $({$ex:expr, $ey:expr}),*) => {{
            let used = vec![$(Region::new($x, $y)),+];

            for (i, r0) in used.iter().enumerate() {
                for (j, r1) in used.iter().enumerate() {
                    if i != j && r0.overlaps(r1) {
                        panic!("invalid data fed into test: {} overlaps with {} - {} overlaps with {}", i, j, r0, r1);
                    }
                }
            }

            let mut expected: Vec<Region<u64>> = vec![$(Region::new($ex, $ey)),*];
            let mut result = find_free_regions(usable_region, &mut used.iter().copied());

            expected.sort_by(|a, b| a.start().cmp(&b.start()));
            result.sort_by(|a, b| a.start().cmp(&b.start()));

            assert_eq!(expected.len(), result.len(), "vector size does not match");

            for (a, b) in expected.iter().zip(result.iter()) {
                assert_eq!(a.start(), b.start());
                assert_eq!(a.end(), b.end());
            }
        }};
    }

    test!({34, 200}, {201, 500} expect {501, 2097116});
    test!({34, 200}, {8192, 16388}, {1048558, 1572837} expect {201, 8191}, {16389,1048557}, {1572838, 2097116});

    test!({201, 500}, {34, 200} expect {501, 2097116});
}
