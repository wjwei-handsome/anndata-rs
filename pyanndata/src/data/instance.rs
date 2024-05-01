use pyo3::{prelude::*, types::PyType, PyResult};

pub fn isinstance_of_csr<'py>(obj: &Bound<'py, PyAny>) -> PyResult<bool> {
    obj.is_instance(
        obj.py().import_bound("scipy.sparse")?
            .getattr("csr_matrix")?
            .downcast::<PyType>()
            .unwrap(),
    )
}

pub fn isinstance_of_csc<'py>(obj: &Bound<'py, PyAny>) -> PyResult<bool> {
    obj.is_instance(
        obj.py().import_bound("scipy.sparse")?
            .getattr("csc_matrix")?
            .downcast::<PyType>()
            .unwrap(),
    )
}

pub fn isinstance_of_arr<'py>(obj: &Bound<'py, PyAny>) -> PyResult<bool> {
    obj.is_instance(
        obj.py().import_bound("numpy")?
            .getattr("ndarray")?
            .downcast::<PyType>()
            .unwrap(),
    )
}

pub fn isinstance_of_pyanndata<'py>(obj: &Bound<'py, PyAny>) -> PyResult<bool> {
    obj.is_instance(
        obj.py().import_bound("anndata")?
            .getattr("AnnData")?
            .downcast::<PyType>()
            .unwrap(),
    )
}

pub fn isinstance_of_pandas<'py>(obj: &Bound<'py, PyAny>) -> PyResult<bool> {
    obj.is_instance(
        obj.py().import_bound("pandas")?
            .getattr("DataFrame")?
            .downcast::<PyType>()
            .unwrap(),
    )
}

pub fn isinstance_of_polars<'py>(obj: &Bound<'py, PyAny>) -> PyResult<bool> {
    obj.is_instance(
        obj.py().import_bound("polars")?
            .getattr("DataFrame")?
            .downcast::<PyType>()
            .unwrap(),
    )
}

pub fn is_none_slice<'py>(obj: &Bound<'py, PyAny>) -> PyResult<bool> {
    let py = obj.py();
    Ok(
        obj.is_none() ||
        obj.is(&py.Ellipsis()) ||
        (is_slice(obj) && obj.eq(py.eval_bound("slice(None, None, None)", None, None)?)?)
    )
}

fn is_slice<'py>(obj: &Bound<'py, PyAny>) -> bool {
    obj.is_instance_of::<pyo3::types::PySlice>()
}