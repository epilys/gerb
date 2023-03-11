/*
 * gerb
 *
 * Copyright 2022 - Manos Pitsidianakis
 *
 * This file is part of gerb.
 *
 * gerb is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * gerb is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with gerb. If not, see <http://www.gnu.org/licenses/>.
 */

//! Wrapper types to expose to Python.
//!
//! They all need a `Py<Gerb>` reference in order to access the
//! API channel from the python thread to the main thread.
use super::*;

macro_rules! get_prop {
    ($ty:ty, $prop:ident) => {{
        Request::ObjectProperty {
            type_name: <$ty>::static_type().name().to_string(),
            kind: Property::Get {
                property: <$ty>::$prop.to_string(),
            },
        }
    }};
}

#[pyclass]
pub struct Project {
    #[pyo3(get)]
    pub(in crate::api) __gerb: Py<Gerb>,
}

macro_rules! getter {
    ($self_:expr, $py: expr, $parent_type:ty, $prop:ident) => {
        $self_
            .__gerb
            .as_ref($py)
            .borrow()
            .__send_rcv(
                serde_json::to_string(&get_prop!($parent_type, $prop)).unwrap(),
                $py,
            )?
            .extract($py)
    };
}

#[pymethods]
impl Project {
    fn __repr__(&self) -> PyResult<String> {
        Ok("Project".to_string())
    }

    /// Return the currently loaded project name.
    #[getter(name)]
    fn name(&self, py: Python<'_>) -> PyResult<String> {
        getter!(self, py, crate::prelude::Project, NAME)
    }

    ///
    #[getter(family_name)]
    fn family_name(&self, py: Python<'_>) -> PyResult<String> {
        getter!(self, py, crate::prelude::Project, FAMILY_NAME)
    }
    ///
    #[getter(style_name)]
    fn style_name(&self, py: Python<'_>) -> PyResult<String> {
        getter!(self, py, crate::prelude::Project, STYLE_NAME)
    }

    ///
    #[getter(style_map_family_name)]
    fn style_map_family_name(&self, py: Python<'_>) -> PyResult<String> {
        getter!(self, py, crate::prelude::Project, STYLE_MAP_FAMILY_NAME)
    }

    #[getter(style_map_family_name)]
    fn style_map_style_name(&self, py: Python<'_>) -> PyResult<String> {
        getter!(self, py, crate::prelude::Project, STYLE_MAP_STYLE_NAME)
    }

    #[getter(copyright)]
    fn copyright(&self, py: Python<'_>) -> PyResult<String> {
        getter!(self, py, crate::prelude::Project, COPYRIGHT)
    }

    #[getter(trademark)]
    fn trademark(&self, py: Python<'_>) -> PyResult<String> {
        getter!(self, py, crate::prelude::Project, TRADEMARK)
    }

    #[getter(note)]
    fn note(&self, py: Python<'_>) -> PyResult<String> {
        getter!(self, py, crate::prelude::Project, NOTE)
    }

    #[getter(year)]
    fn year(&self, py: Python<'_>) -> PyResult<u64> {
        getter!(self, py, crate::prelude::Project, YEAR)
    }

    ///
    #[getter(modified)]
    fn modified(&self, py: Python<'_>) -> PyResult<bool> {
        getter!(self, py, crate::prelude::Project, MODIFIED)
    }

    #[getter(version_major)]
    fn version_major(&self, py: Python<'_>) -> PyResult<i64> {
        getter!(self, py, crate::prelude::Project, VERSION_MAJOR)
    }

    #[getter(version_minor)]
    fn version_minor(&self, py: Python<'_>) -> PyResult<u64> {
        getter!(self, py, crate::prelude::Project, VERSION_MINOR)
    }

    #[getter(units_per_em)]
    fn units_per_em(&self, py: Python<'_>) -> PyResult<f64> {
        getter!(self, py, crate::prelude::Project, UNITS_PER_EM)
    }

    #[getter(x_height)]
    fn x_height(&self, py: Python<'_>) -> PyResult<f64> {
        getter!(self, py, crate::prelude::Project, X_HEIGHT)
    }

    #[getter(ascender)]
    fn ascender(&self, py: Python<'_>) -> PyResult<f64> {
        getter!(self, py, crate::prelude::Project, ASCENDER)
    }

    #[getter(descender)]
    fn descender(&self, py: Python<'_>) -> PyResult<f64> {
        getter!(self, py, crate::prelude::Project, DESCENDER)
    }

    #[getter(cap_height)]
    fn cap_height(&self, py: Python<'_>) -> PyResult<f64> {
        getter!(self, py, crate::prelude::Project, CAP_HEIGHT)
    }

    #[getter(italic_angle)]
    fn italic_angle(&self, py: Python<'_>) -> PyResult<f64> {
        getter!(self, py, crate::prelude::Project, ITALIC_ANGLE)
    }
}
