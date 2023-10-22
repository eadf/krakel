use super::{KDTree, PointTrait};
use std::cmp::Reverse;
use vector_traits::glam::{dvec2, vec2};
use vector_traits::{
    approx::ulps_eq,
    glam::{DVec2, Vec2},
};

#[derive(Default)]
pub struct MaximumTracker<T> {
    current_max: Option<T>,
}

impl<T: PartialOrd + Copy> MaximumTracker<T> {
    #[inline(always)]
    fn insert(&mut self, value: T) {
        match self.current_max {
            Some(max_value) if value > max_value => self.current_max = Some(value),
            None => self.current_max = Some(value),
            _ => (),
        }
    }
    #[inline(always)]
    pub fn get_max(&self) -> Option<T> {
        self.current_max
    }
}
fn assert_approx_eq<T: ApproxEq + std::fmt::Debug>(v1: T, v2: T) {
    assert!(v1.approx_eq(&v2), "{:?} != {:?}", v1, v2);
}

trait ApproxEq {
    fn approx_eq(&self, other: &Self) -> bool;
}

impl ApproxEq for DVec2 {
    fn approx_eq(&self, other: &Self) -> bool {
        ulps_eq!(self.x, other.x) && ulps_eq!(self.y, other.y)
    }
}

impl ApproxEq for Vec2 {
    fn approx_eq(&self, other: &Self) -> bool {
        ulps_eq!(self.x, other.x) && ulps_eq!(self.y, other.y)
    }
}

#[test]
fn test_nearest_query() {
    use crate::PointTrait;
    let mut kdtree = KDTree::<DVec2>::default();

    let points = vec![
        DVec2 { x: 2.0, y: 3.0 },
        DVec2 { x: 5.0, y: 4.0 },
        DVec2 { x: 9.0, y: 6.0 },
        DVec2 { x: 4.0, y: 7.0 },
        DVec2 { x: 8.0, y: 1.0 },
        DVec2 { x: 7.0, y: 2.0 },
    ];

    for point in &points {
        println!("inserting {:?}", point);
        kdtree.insert(point.clone()).unwrap();
    }

    //kdtree.print_tree();

    let search_point = DVec2 { x: 7.5, y: 3.5 };
    println!("before nearest");
    let nearest = kdtree.nearest(&search_point);
    println!("after nearest");

    match nearest {
        Some(pt) => {
            println!(
                "The nearest point to {:?} is {:?} dist={}.",
                search_point,
                pt,
                PointTrait::dist_sq(&search_point, &pt).sqrt()
            );
            let mut m = MaximumTracker::<Reverse<f64>>::default();
            for p in &points {
                m.insert(Reverse(PointTrait::dist_sq(&search_point, p).sqrt()));
                println!(
                    "testing {:?} dist = {}",
                    p,
                    PointTrait::dist_sq(&search_point, p)
                );
            }
            println!(
                "Actual closest point is distance {:?}",
                m.get_max().unwrap().0
            );
            assert_eq!(
                m.get_max().unwrap().0,
                PointTrait::dist_sq(&search_point, &pt).sqrt()
            );
        }
        None => panic!("No point found in the KDTree!"),
    }
}

#[test]
fn test_nearest_query_2() {
    use crate::PointTrait;
    use rand::{rngs::StdRng, Rng, SeedableRng};
    let mut kdtree = KDTree::<DVec2>::default();
    // Initialize a deterministic RNG with a fixed seed
    let mut rng: StdRng = SeedableRng::seed_from_u64(42);

    let mut points = vec![DVec2 { x: 2.0, y: 3.0 }];
    // Add random points
    for _ in 0..3000 {
        // Adds 3000 random points; adjust as needed
        points.push(DVec2 {
            x: rng.gen_range(0.0..10.0), // Random x between 0 and 10
            y: rng.gen_range(0.0..10.0), // Random y between 0 and 10
        });
    }

    for point in &points {
        //println!("inserting {:?}", point);
        kdtree.insert(point.clone()).unwrap();
    }

    //kdtree.print_tree();

    let search_point = DVec2 { x: 7.5, y: 3.5 };
    //println!("before nearest");
    let nearest = kdtree.nearest(&search_point);
    //println!("after nearest");

    match nearest {
        Some(pt) => {
            /*println!(
                "The nearest point to {:?} is {:?} dist={}.",
                search_point,
                pt,
                search_point.dist_sq(&pt).sqrt()
            );*/
            let mut m = MaximumTracker::<Reverse<f64>>::default();
            for p in &points {
                m.insert(Reverse(PointTrait::dist_sq(&search_point, p).sqrt()));
                //println!("testing {:?} dist = {}", p, search_point.dist_sq(&p));
            }
            /*println!(
                "Actual closest point is distance {:?}",
                m.get_max().unwrap().0
            );
             */
            assert_eq!(
                m.get_max().unwrap().0,
                PointTrait::dist_sq(&search_point, &pt).sqrt()
            );
        }
        None => panic!("No point found in the KDTree!"),
    }

    for search_point in &points {
        let results = kdtree.nearest(search_point);
        assert_approx_eq(*search_point, results.unwrap());
    }
}

#[test]
fn test_range_query() {
    use crate::PointTrait;
    let mut kdtree = KDTree::<DVec2>::default();

    let points = vec![
        DVec2 { x: 2.0, y: 3.0 },
        DVec2 { x: 5.0, y: 4.0 },
        DVec2 { x: 9.0, y: 6.0 },
        DVec2 { x: 4.0, y: 7.0 },
        DVec2 { x: 8.0, y: 1.0 },
        DVec2 { x: 7.0, y: 2.0 },
    ];

    for point in &points {
        kdtree.insert(point.clone()).unwrap();
    }

    let search_point = DVec2 { x: 7.5, y: 3.5 };
    let radius = 2.93;

    println!("{:?}", kdtree);

    let results = kdtree.range_query(&search_point, radius);

    // Let's test if the results are within the given range and if the results are correct.
    for pt in &results {
        let dist = PointTrait::dist_sq(&search_point, pt).sqrt();
        assert!(
            dist <= radius,
            "found distance :{:?}, expected to be less than {:?}",
            dist,
            radius
        );
    }
    println!(
        "found points {:?} should all be less than distance:{}",
        results, radius
    );

    // Based on the provided points and the search_point, we can expect the following points
    // to be in the result: (8.0, 1.0) and (7.0, 2.0)
    let expected_points = vec![DVec2 { x: 8.0, y: 1.0 }, DVec2 { x: 7.0, y: 2.0 }];

    for expected in &expected_points {
        assert!(results.contains(expected));
    }

    for search_point in &points {
        let results = kdtree.range_query(search_point, 0.0001);
        assert!(results.contains(search_point));
    }
}

#[test]
fn test_range_query_2() {
    use crate::PointTrait;
    use rand::{rngs::StdRng, Rng, SeedableRng};

    let mut kdtree = KDTree::<DVec2>::default();
    // Initialize a deterministic RNG with a fixed seed
    let mut rng: StdRng = SeedableRng::seed_from_u64(42);

    let mut points = vec![
        DVec2 { x: 2.0, y: 3.0 },
        DVec2 { x: 8.0, y: 1.0 },
        DVec2 { x: 7.0, y: 2.0 },
    ];
    // Add random points
    for _ in 0..300 {
        // Adds 300 random points; adjust as needed
        points.push(DVec2 {
            x: rng.gen_range(0.0..10.0), // Random x between 0 and 10
            y: rng.gen_range(0.0..10.0), // Random y between 0 and 10
        });
    }

    for point in &points {
        kdtree.insert(point.clone()).unwrap();
    }

    let search_radius = 0.01;
    for search_point in &points {
        let results = kdtree.range_query(search_point, search_radius);
        assert!(results.contains(search_point));
        for pt in &results {
            let dist = PointTrait::dist_sq(search_point, pt).sqrt();
            assert!(
                dist <= search_radius,
                "found distance :{:?}, expected to be less than {:?}",
                dist,
                search_radius
            );
        }
    }
}

#[test]
fn test_range_query_3() {
    use crate::PointTrait;
    use rand::{rngs::StdRng, Rng, SeedableRng};

    let mut kdtree = KDTree::<DVec2>::default();
    // Initialize a deterministic RNG with a fixed seed
    let mut rng: StdRng = SeedableRng::seed_from_u64(42);

    let mut points = vec![
        DVec2 { x: 2.0, y: 3.0 },
        DVec2 { x: 8.0, y: 1.0 },
        DVec2 { x: 7.0, y: 2.0 },
    ];
    // Add random points
    for _ in 0..300 {
        // Adds 300 random points; adjust as needed
        points.push(DVec2 {
            x: rng.gen_range(0.0..10.0), // Random x between 0 and 10
            y: rng.gen_range(0.0..10.0), // Random y between 0 and 10
        });
    }

    for point in &points {
        kdtree.insert(point.clone()).unwrap();
    }

    let search_radius = 0.01;

    for search_point in &points {
        let mut mmax = MaximumTracker::<f64>::default();
        kdtree.closure_range_query(search_point, search_radius, |site| {
            let dist = PointTrait::dist_sq(search_point, site).sqrt();
            mmax.insert(dist);
            assert!(
                dist <= search_radius,
                "found distance :{:?}, expected to be less than {:?}",
                dist,
                search_radius
            );
        });
        let mmax = mmax.get_max().unwrap();
        assert!(
            mmax <= search_radius,
            "found distance :{:?}, expected to be less than {:?}",
            mmax,
            search_radius
        );
    }
}

#[test]
fn test_range_query_4_dvec2() {
    use crate::PointTrait;
    use rand::{rngs::StdRng, Rng, SeedableRng};

    let mut kdtree = KDTree::<DVec2>::default();
    // Initialize a deterministic RNG with a fixed seed
    let mut rng: StdRng = SeedableRng::seed_from_u64(42);

    let mut points = vec![
        DVec2 { x: 2.0, y: 3.0 },
        DVec2 { x: 8.0, y: 1.0 },
        DVec2 { x: 7.0, y: 2.0 },
    ];
    // Add random points
    for _ in 0..3000 {
        // Adds 300 random points; adjust as needed
        points.push(DVec2 {
            x: rng.gen_range(0.0..10.0), // Random x between 0 and 10
            y: rng.gen_range(0.0..10.0), // Random y between 0 and 10
        });
    }

    for point in &points {
        kdtree.insert(point.clone()).unwrap();
    }

    let search_radius = 0.01;
    let offset = DVec2 { x: 1.0, y: 1.0 }.normalize() * search_radius * 0.99995;

    for search_point in &points {
        let mut mmax = MaximumTracker::<f64>::default();
        let offset_search_position = &(*search_point + offset);

        kdtree.closure_range_query(offset_search_position, search_radius, |site| {
            let dist = PointTrait::dist_sq(search_point, site).sqrt();
            if search_point.approx_eq(site) {
                mmax.insert(dist);
                assert!(
                    dist <= search_radius,
                    "found distance :{:?}, expected to be less than {:?}",
                    dist,
                    search_radius
                );
            } else {
                let real_dist = PointTrait::dist_sq(offset_search_position, site).sqrt();
                assert!(
                    real_dist <= search_radius,
                    "Found a match outside the search radius. dist:{} search_radius:{}",
                    real_dist,
                    search_radius
                );
            }
        });
        assert!(
            mmax.get_max().is_some(),
            "The expected sample was not found"
        );
        let mmax = mmax.get_max().unwrap();
        assert!(
            mmax <= search_radius,
            "found distance :{:?}, expected to be less than {:?}",
            mmax,
            search_radius
        );
    }
}

#[test]
fn test_range_query_4_vec2() {
    let mut a_point = vec2(1.0, 2.0);
    a_point.set_x(4.0);
    a_point.set_x(8.0);

    use crate::PointTrait;
    use rand::{rngs::StdRng, Rng, SeedableRng};

    let mut kdtree = KDTree::<Vec2>::default();
    // Initialize a deterministic RNG with a fixed seed
    let mut rng: StdRng = SeedableRng::seed_from_u64(42);

    let mut points = vec![
        Vec2 { x: 2.0, y: 3.0 },
        Vec2 { x: 8.0, y: 1.0 },
        Vec2 { x: 7.0, y: 2.0 },
    ];
    // Add random points
    for _ in 0..3000 {
        // Adds 300 random points; adjust as needed
        points.push(Vec2 {
            x: rng.gen_range(0.0..10.0), // Random x between 0 and 10
            y: rng.gen_range(0.0..10.0), // Random y between 0 and 10
        });
    }

    for point in &points {
        kdtree.insert(point.clone()).unwrap();
    }

    let search_radius = 0.01;
    let offset = Vec2 { x: 1.0, y: 1.0 }.normalize() * search_radius * 0.99995;

    for search_point in &points {
        let mut mmax = MaximumTracker::<f32>::default();
        let offset_search_position = &(*search_point + offset);

        kdtree.closure_range_query(offset_search_position, search_radius, |site| {
            let dist = PointTrait::dist_sq(search_point, site).sqrt();
            if search_point.approx_eq(site) {
                mmax.insert(dist);
                assert!(
                    dist <= search_radius,
                    "found distance :{:?}, expected to be less than {:?}",
                    dist,
                    search_radius
                );
            } else {
                let real_dist = PointTrait::dist_sq(offset_search_position, site).sqrt();
                assert!(
                    real_dist <= search_radius,
                    "Found a match outside the search radius. dist:{} search_radius:{}",
                    real_dist,
                    search_radius
                );
            }
        });
        assert!(
            mmax.get_max().is_some(),
            "The expected sample was not found"
        );
        let mmax = mmax.get_max().unwrap();
        assert!(
            mmax <= search_radius,
            "found distance :{:?}, expected to be less than {:?}",
            mmax,
            search_radius
        );
    }
}

#[test]
fn test_set_x_set_y() {
    // test the set_x() and set_y() of the Pointtrait impls
    let mut a_point = vec2(1.0, 2.0);
    a_point.set_x(4.0);
    a_point.set_y(8.0);
    assert_eq!(a_point.x(), 4.0);
    assert_eq!(a_point.y(), 8.0);

    let mut a_point = dvec2(1.0, 2.0);
    a_point.set_x(4.0);
    a_point.set_y(8.0);
    assert_eq!(a_point.x(), 4.0);
    assert_eq!(a_point.y(), 8.0);
}
