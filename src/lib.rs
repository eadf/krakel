/*  SPDX-License-Identifier:LGPL-2.0-only
 *  Rust code Copyright (c) 2023 lacklustr@protonmail.com https://github.com/eadf
 *
 *  This file is ported from code inside of OpenCAMlib:
 *  Copyright (c) 2010-2011 Anders Wallin (anders.e.e.wallin "at" gmail.com).
 *  (see https://github.com/aewallin/opencamlib).
 *
 *  This program is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU Lesser General Public License as published by
 *  the Free Software Foundation, either version 2.1 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU Lesser General Public License for more details.
 *
 *  You should have received a copy of the GNU Lesser General Public License
 *  along with this program. If not, see <http://www.gnu.org/licenses/>.
*/

use std::fmt::Display;
use std::{
    fmt,
    fmt::Debug,
    ops::{AddAssign, DivAssign, MulAssign, SubAssign},
};
use approx::UlpsEq;
use num_traits::{real::Real, FromPrimitive, Zero};

mod impls;

#[cfg(test)]
mod tests;

#[derive(thiserror::Error, Debug)]
pub enum KrakelError {
    #[error("Unknown error: {0}")]
    InternalError(String),
}

pub trait PointTrait: Clone + PartialEq
where
    Self::PScalar: Real
        + FromPrimitive
        + UlpsEq
        + Debug
        + Display
        + PartialEq
        + MulAssign
        + SubAssign
        + DivAssign
        + AddAssign,
{
    type PScalar;
    fn x(&self) -> Self::PScalar;
    fn y(&self) -> Self::PScalar;
    fn set_x(&mut self, x: Self::PScalar);
    fn set_y(&mut self, y: Self::PScalar);

    /// Returns the squared distance between this point and another point that is using the same scalar type.
    #[inline(always)]
    fn dist_sq<Q: PointTrait<PScalar = Self::PScalar>>(a: &Self, b: &Q) -> Self::PScalar {
        let dx: Self::PScalar = a.x() - b.x();
        let dy: Self::PScalar = a.y() - b.y();
        dx * dx + dy * dy
    }

    fn at(&self, index: u8) -> Self::PScalar;
    fn at_mut(&mut self, index: u8) -> &mut Self::PScalar;
    const DIMENSION: u8;
}

pub trait KDPoint<P: PointTrait> {
    fn get_coordinate(&self, index: usize) -> P::PScalar;
    fn set_coordinate(&mut self, index: usize, value: P::PScalar);
}

pub struct KDNode<P: PointTrait> {
    pos: P,
    dir: u8,
    left: Option<Box<KDNode<P>>>,
    right: Option<Box<KDNode<P>>>,
}

#[derive(Clone)]
struct HyperRectangle<P: PointTrait> {
    min: P,
    max: P,
}

pub struct KDTree<P: PointTrait> {
    root: Option<Box<KDNode<P>>>,
    rect: Option<HyperRectangle<P>>,
}

impl<P: PointTrait> KDNode<P> {
    fn recursive_insert(
        node: &mut Option<Box<KDNode<P>>>,
        pos: P,
        dir: u8,
        dim: u8,
    ) -> Result<(), KrakelError> {
        match node {
            None => {
                *node = Some(Box::new(KDNode {
                    pos,
                    dir,
                    left: None,
                    right: None,
                }));
            }
            Some(current) => {
                let new_dir = (current.dir + 1) % dim;
                if pos.at(current.dir) < current.pos.at(current.dir) {
                    Self::recursive_insert(&mut current.left, pos, new_dir, dim)?;
                } else {
                    Self::recursive_insert(&mut current.right, pos, new_dir, dim)?;
                }
            }
        }
        Ok(())
    }

    fn recursive_nearest<'a>(
        &'a self,
        pos: &P,
        result: &mut Option<&'a P>,
        result_dist_sq: &mut P::PScalar,
        rect: &mut HyperRectangle<P>,
    ) {
        let dir = self.dir;

        let (nearer_subtree, farther_subtree) = if pos.at(dir) <= self.pos.at(dir) {
            (&self.left, &self.right)
        } else {
            (&self.right, &self.left)
        };

        let old_value = if pos.at(dir) <= self.pos.at(dir) {
            std::mem::replace(&mut rect.max.at(dir), self.pos.at(dir))
        } else {
            std::mem::replace(&mut rect.min.at(dir), self.pos.at(dir))
        };

        if let Some(nearer_node) = nearer_subtree {
            nearer_node.recursive_nearest(pos, result, result_dist_sq, rect);
        }

        if pos.at(dir) <= self.pos.at(dir) {
            *rect.max.at_mut(dir) = old_value;
        } else {
            *rect.min.at_mut(dir) = old_value;
        }

        let dist_sq = PointTrait::dist_sq(&self.pos, pos);
        if dist_sq < *result_dist_sq {
            *result_dist_sq = dist_sq;
            *result = Some(&self.pos);
        }

        if let Some(farther_node) = farther_subtree {
            if KDTree::hyper_rect_dist_sq(rect, pos) < *result_dist_sq {
                farther_node.recursive_nearest(pos, result, result_dist_sq, rect);
            }
        }
    }

    fn recursive_range_query<Q: PointTrait<PScalar = P::PScalar>>(
        &self,
        pos: &Q,
        radius_sq: P::PScalar,
        results: &mut Vec<P>,
        rect: &mut HyperRectangle<P>,
    ) {
        let dir = self.dir;

        let (nearer_subtree, farther_subtree) = if pos.at(dir) <= self.pos.at(dir) {
            (&self.left, &self.right)
        } else {
            (&self.right, &self.left)
        };

        let old_value = if pos.at(dir) <= self.pos.at(dir) {
            std::mem::replace(&mut rect.max.at(dir), self.pos.at(dir))
        } else {
            std::mem::replace(&mut rect.min.at(dir), self.pos.at(dir))
        };

        if let Some(nearer_node) = nearer_subtree {
            nearer_node.recursive_range_query(pos, radius_sq, results, rect);
        }

        if pos.at(dir) <= self.pos.at(dir) {
            *rect.max.at_mut(dir) = old_value;
        } else {
            *rect.min.at_mut(dir) = old_value;
        }

        let dist_sq = PointTrait::dist_sq(&self.pos, pos);
        if dist_sq <= radius_sq {
            results.push(self.pos.clone());
        }

        if let Some(farther_node) = farther_subtree {
            if KDTree::hyper_rect_dist_sq(rect, pos) <= radius_sq {
                farther_node.recursive_range_query(pos, radius_sq, results, rect);
            }
        }
    }

    fn recursive_closure_range_query<Q: PointTrait<PScalar = P::PScalar>, F>(
        &self,
        pos: &Q,
        radius_sq: P::PScalar,
        rect: &mut HyperRectangle<P>,
        process: &mut F,
    ) where
        F: FnMut(&P),
    {
        let dir = self.dir;

        let (nearer_subtree, farther_subtree) = if pos.at(dir) <= self.pos.at(dir) {
            (&self.left, &self.right)
        } else {
            (&self.right, &self.left)
        };

        let old_value = if pos.at(dir) <= self.pos.at(dir) {
            std::mem::replace(&mut rect.max.at(dir), self.pos.at(dir))
        } else {
            std::mem::replace(&mut rect.min.at(dir), self.pos.at(dir))
        };

        if let Some(nearer_node) = nearer_subtree {
            nearer_node.recursive_closure_range_query(pos, radius_sq, rect, process);
        }

        if pos.at(dir) <= self.pos.at(dir) {
            *rect.max.at_mut(dir) = old_value;
        } else {
            *rect.min.at_mut(dir) = old_value;
        }

        if PointTrait::dist_sq(&self.pos, pos) <= radius_sq {
            process(&self.pos);
        }

        if let Some(farther_node) = farther_subtree {
            if KDTree::hyper_rect_dist_sq(rect, pos) <= radius_sq {
                farther_node.recursive_closure_range_query(pos, radius_sq, rect, process);
            }
        }
    }

    fn format_node(&self, f: &mut fmt::Formatter<'_>, depth: usize) -> fmt::Result {
        for _ in 0..depth {
            write!(f, " ")?;
        }

        write!(f, "d={} node at ", self.dir)?;
        for i in 0..P::DIMENSION {
            write!(f, "{} ", self.pos.at(i))?;
        }
        writeln!(f)?;

        if let Some(ref left_node) = self.left {
            left_node.format_node(f, depth + 1)?;
        }

        if let Some(ref right_node) = self.right {
            right_node.format_node(f, depth + 1)?;
        }

        Ok(())
    }
}

impl<P: PointTrait> KDTree<P> {
    pub fn insert(&mut self, pos: P) -> Result<(), KrakelError> {
        KDNode::recursive_insert(&mut self.root, pos.clone(), 0, P::DIMENSION)?;

        if self.rect.is_none() {
            self.rect = Some(HyperRectangle {
                min: pos.clone(),
                max: pos.clone(),
            });
        } else {
            for i in 0..P::DIMENSION {
                if pos.at(i) < self.rect.as_mut().unwrap().min.at(i) {
                    *self.rect.as_mut().unwrap().min.at_mut(i) = pos.at(i);
                } else if pos.at(i) > self.rect.as_mut().unwrap().max.at(i) {
                    *self.rect.as_mut().unwrap().max.at_mut(i) = pos.at(i);
                }
            }
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub fn nearest(&self, pos: &P) -> Option<P> {
        if let Some(root_node) = &self.root {
            // Now that we know self.root is Some(_), it's safe to assume self.rect is Some(_) as well
            let mut rect = self.rect.clone().unwrap();
            let mut result: Option<&P> = self.root.as_ref().map(|node| &node.pos);
            let mut result_dist_sq = P::dist_sq(result.as_ref().unwrap(), pos);

            root_node.recursive_nearest(pos, &mut result, &mut result_dist_sq, &mut rect);
            result.cloned()
        } else {
            None
        }
    }

    #[allow(dead_code)]
    pub fn range_query<Q: PointTrait<PScalar = P::PScalar>>(
        &self,
        pos: &Q,
        radius: P::PScalar,
    ) -> Vec<P> {
        if let Some(root_node) = &self.root {
            let mut results: Vec<P> = Vec::new();
            let mut cloned_rect = self.rect.clone().unwrap();

            root_node.recursive_range_query(pos, radius * radius, &mut results, &mut cloned_rect);
            results
        } else {
            Vec::new()
        }
    }

    pub fn closure_range_query<Q: PointTrait<PScalar = P::PScalar>, F>(
        &self,
        pos: &Q,
        radius: P::PScalar,
        mut process: F,
    ) where
        F: FnMut(&P),
    {
        if let Some(root_node) = &self.root {
            let mut cloned_rect = self.rect.clone().unwrap();

            root_node.recursive_closure_range_query(
                pos,
                radius * radius,
                &mut cloned_rect,
                &mut process,
            );
        }
    }

    fn hyper_rect_dist_sq<Q: PointTrait<PScalar = P::PScalar>>(
        rect: &HyperRectangle<P>,
        pos: &Q,
    ) -> P::PScalar {
        let mut result = P::PScalar::zero();
        for i in 0..P::DIMENSION {
            let pos_val = pos.at(i);
            if pos_val < rect.min.at(i) {
                result += Self::sq(rect.min.at(i) - pos_val);
            } else if pos_val > rect.max.at(i) {
                result += Self::sq(rect.max.at(i) - pos_val);
            }
        }
        result
    }

    #[inline(always)]
    fn sq(i: P::PScalar) -> P::PScalar {
        i * i
    }
}
