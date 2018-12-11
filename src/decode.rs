// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use pyo3::prelude::*;
use pyo3::types::*;
use pyo3::IntoPyPointer;
use serde::de::{self, DeserializeSeed, Deserializer, MapAccess, SeqAccess, Visitor};
use smallvec::SmallVec;
use std::borrow::Cow;
use std::fmt;
use std::marker::PhantomData;

import_exception!(json, JSONDecodeError);

pub fn deserialize(py: Python, data: &str) -> PyResult<PyObject> {
    let seed = JsonValue::new(py);
    let mut deserializer = serde_json::Deserializer::from_str(data);
    match seed.deserialize(&mut deserializer) {
        Ok(py_ptr) => {
            deserializer
                .end()
                .map_err(|e| JSONDecodeError::py_err((e.to_string(), "", 0)))?;
            Ok(unsafe { PyObject::from_owned_ptr(py, py_ptr) })
        }
        Err(e) => {
            return Err(JSONDecodeError::py_err((e.to_string(), "", 0)));
        }
    }
}

#[derive(Clone)]
struct JsonValue<'a> {
    py: Python<'a>,
}

impl<'a> JsonValue<'a> {
    fn new(py: Python<'a>) -> JsonValue<'a> {
        JsonValue { py }
    }
}

impl<'de, 'a> DeserializeSeed<'de> for JsonValue<'a> {
    type Value = *mut pyo3::ffi::PyObject;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
}

impl<'de, 'a> Visitor<'de> for JsonValue<'a> {
    type Value = *mut pyo3::ffi::PyObject;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("JSON")
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(unsafe { pyo3::ffi::Py_None() })
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(value.into_object(self.py).into_ptr())
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(value.into_object(self.py).into_ptr())
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(value.into_object(self.py).into_ptr())
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(PyFloat::new(self.py, value).into_ptr())
    }

    fn visit_borrowed_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(PyString::new(self.py, value).into_ptr())
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(PyString::new(self.py, value).into_ptr())
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut elements: SmallVec<[*mut pyo3::ffi::PyObject; 8]> = SmallVec::new();
        while let Some(elem) = seq.next_element_seed(self.clone())? {
            elements.push(elem);
        }
        let ptr = unsafe { pyo3::ffi::PyList_New(elements.len() as pyo3::ffi::Py_ssize_t) };
        for (i, obj) in elements.iter().enumerate() {
            unsafe { pyo3::ffi::PyList_SetItem(ptr, i as pyo3::ffi::Py_ssize_t, *obj) };
        }
        Ok(ptr)
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let dict_ptr = PyDict::new(self.py).into_ptr();
        while let Some((key, value)) = map.next_entry_seed(PhantomData::<Cow<str>>, self.clone())? {
            let _ = unsafe { pyo3::ffi::PyDict_SetItem(
                dict_ptr,
                PyString::new(self.py, &key).into_ptr(),
                value,
            ) };
        }
        Ok(dict_ptr)
    }
}