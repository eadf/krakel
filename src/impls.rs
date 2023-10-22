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

use super::*;
use std::fmt;
#[cfg(feature = "glam")]
use vector_traits::glam::{DVec2, Vec2};

#[cfg(feature = "glam")]
impl PointTrait for Vec2 {
    type PScalar = f32;
    #[inline(always)]
    fn x(&self) -> Self::PScalar {
        self.x
    }
    #[inline(always)]
    fn y(&self) -> Self::PScalar {
        self.y
    }
    #[inline(always)]
    fn set_x(&mut self, x: Self::PScalar) {
        self.x = x;
    }
    #[inline(always)]
    fn set_y(&mut self, y: Self::PScalar) {
        self.y = y;
    }
    #[inline(always)]
    fn at(&self, index: u8) -> Self::PScalar {
        match index {
            0 => self.x,
            1 => self.y,
            _ => unreachable!(),
        }
    }
    #[inline(always)]
    fn at_mut(&mut self, index: u8) -> &mut Self::PScalar {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            _ => unreachable!(),
        }
    }

    const DIMENSION: u8 = 2;
}

#[cfg(feature = "glam")]
impl PointTrait for DVec2 {
    type PScalar = f64;
    #[inline(always)]
    fn x(&self) -> Self::PScalar {
        self.x
    }
    #[inline(always)]
    fn y(&self) -> Self::PScalar {
        self.y
    }
    #[inline(always)]
    fn set_x(&mut self, x: Self::PScalar) {
        self.x = x;
    }
    #[inline(always)]
    fn set_y(&mut self, y: Self::PScalar) {
        self.y = y;
    }

    #[inline(always)]
    fn at(&self, index: u8) -> Self::PScalar {
        match index {
            0 => self.x,
            1 => self.y,
            _ => unreachable!(),
        }
    }
    #[inline(always)]
    fn at_mut(&mut self, index: u8) -> &mut Self::PScalar {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            _ => unreachable!(),
        }
    }

    const DIMENSION: u8 = 2;
}

impl<P: PointTrait> Default for KDTree<P> {
    fn default() -> Self {
        Self {
            root: None,
            rect: None,
        }
    }
}

impl<P: PointTrait> Debug for KDTree<P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref root_node) = self.root {
            writeln!(f, "KDTree(")?;
            root_node.format_node(f, 0)?;
            writeln!(f, ")")
        } else {
            writeln!(f, "KDTree()")
        }
    }
}
