pub mod element;
pub mod dataset;
pub mod iterator;
pub mod utils;
use element::{
    PyElemCollection, PyAxisArrays,
    PyMatrixElemOptional, PyDataFrameElem,
};

use utils::{
    conversion::{to_py_df, to_rust_df, to_rust_data1, to_rust_data2, to_py_data2},
    instance::{isinstance_of_arr, isinstance_of_pandas},
};

use anndata_rs::base;
use pyo3::{
    prelude::*,
    PyResult, Python,
};
use std::collections::HashMap;
use std::fmt::Write;

#[pyclass]
#[repr(transparent)]
#[derive(Clone)]
pub struct AnnData(pub base::AnnData);

#[pymethods]
impl AnnData {
    #[new]
    #[args(
        "*",
        filename,
        X = "None",
        n_obs = "None",
        n_vars = "None",
        obs = "None",
        var = "None",
        obsm = "None",
    )]
    fn new<'py>(
        py: Python<'py>,
        filename: &str,
        X: Option<&'py PyAny>,
        n_obs: Option<usize>,
        n_vars: Option<usize>,
        obs: Option<&'py PyAny>,
        var: Option<&'py PyAny>,
        obsm: Option<HashMap<String, &'py PyAny>>,
    ) -> PyResult<Self> {
        let mut anndata = AnnData(base::AnnData::new(
            filename, n_obs.unwrap_or(0), n_vars.unwrap_or(0)
        ).unwrap());
       if let Some(d) = X { anndata.set_x(py, d)?; }
       if let Some(d) = obs { anndata.set_obs(py, d)?; }
       if let Some(d) = var { anndata.set_var(py, d)?; }
       if let Some(d) = obsm { anndata.set_obsm(py, d)?; }
       Ok(anndata)
    }

    #[getter]
    fn shape(&self) -> (usize, usize) { (self.n_obs(), self.n_vars()) }

    #[getter]
    fn n_obs(&self) -> usize { self.0.n_obs() }

    #[getter]
    fn n_vars(&self) -> usize { self.0.n_vars() }

    #[getter]
    fn var_names(&self) -> PyObject {
        todo!()
    }

    #[getter]
    fn obs_names(&self) -> PyObject {
        todo!()
    }

    #[getter(X)]
    fn get_x(&self) -> Option<PyMatrixElemOptional> {
        if self.0.x.is_empty() {
            None
        } else {
            Some(PyMatrixElemOptional(self.0.x.clone()))
        }
    }

    #[setter(X)]
    fn set_x<'py>(&self, py: Python<'py>, data: &'py PyAny) -> PyResult<()> {
            self.0.set_x(&to_rust_data2(py, data)?).unwrap();
            Ok(())
    }

    #[getter(obs)]
    fn get_obs(&self) -> Option<PyDataFrameElem> {
        if self.0.obs.is_empty() {
            None
        } else {
            Some(PyDataFrameElem(self.0.obs.clone()))
        }
    }

    #[setter(obs)]
    fn set_obs<'py>(&self, py: Python<'py>, df: &'py PyAny) -> PyResult<()> {
        let polars = py.import("polars")?;
        let df_ = if isinstance_of_pandas(py, df)? {
            polars.call_method1("from_pandas", (df, ))?
        } else if df.is_instance_of::<pyo3::types::PyDict>()? {
            polars.call_method1("from_dict", (df, ))?
        } else {
            df
        };
 
        self.0.set_obs(&to_rust_df(df_)?).unwrap();
        Ok(())
    }



    #[getter(obsm)]
    fn get_obsm(&self) -> PyAxisArrays { PyAxisArrays(self.0.obsm.clone()) }

    #[setter(obsm)]
    fn set_obsm<'py>(&mut self, py: Python<'py>, mut obsm: HashMap<String, &'py PyAny>) -> PyResult<()> {
        let obsm_: PyResult<_> = obsm.drain().map(|(k, v)|
            Ok((k, to_rust_data2(py, v)?))
        ).collect();
        self.0.set_obsm(&obsm_?).unwrap();
        Ok(())
    }
    
    #[getter(obsp)]
    fn get_obsp(&self) -> PyAxisArrays { PyAxisArrays(self.0.obsp.clone()) }

    #[setter(obsp)]
    fn set_obsp<'py>(&mut self, py: Python<'py>, mut obsp: HashMap<String, &'py PyAny>) {
        let obsp_ = obsp.drain().map(|(k, v)| (k, to_rust_data2(py, v).unwrap())).collect();
        self.0.set_obsp(&obsp_).unwrap();
    }
    
    #[getter(var)]
    fn get_var(&self) -> Option<PyDataFrameElem> {
        if self.0.var.is_empty() {
            None
        } else {
            Some(PyDataFrameElem(self.0.var.clone()))
        }
    }

    #[setter(var)]
    fn set_var<'py>(&self, py: Python<'py>, df: &'py PyAny) -> PyResult<()> {
        let polars = py.import("polars")?;
        let df_ = if isinstance_of_pandas(py, df)? {
            polars.call_method1("from_pandas", (df, ))?
        } else if df.is_instance_of::<pyo3::types::PyDict>()? {
            polars.call_method1("from_dict", (df, ))?
        } else {
            df
        };
 
        self.0.set_var(&to_rust_df(df_)?).unwrap();
        Ok(())
    }

    #[getter(varm)]
    fn get_varm(&self) -> PyAxisArrays { PyAxisArrays(self.0.varm.clone()) }

    #[setter(varm)]
    fn set_varm<'py>(&mut self, py: Python<'py>, mut varm: HashMap<String, &'py PyAny>) {
        let varm_ = varm.drain().map(|(k, v)| (k, to_rust_data2(py, v).unwrap())).collect();
        self.0.set_varm(&varm_).unwrap();
    }

    #[getter(varp)]
    fn get_varp(&self) -> PyAxisArrays { PyAxisArrays(self.0.varp.clone()) }
    
    #[setter(varp)]
    fn set_varp<'py>(&mut self, py: Python<'py>, mut varp: HashMap<String, &'py PyAny>)
    {
        let varp_ = varp.drain().map(|(k, v)| (k, to_rust_data2(py, v).unwrap())).collect();
        self.0.set_varp(&varp_).unwrap();
    }
    
    #[getter(uns)]
    fn get_uns(&self) -> PyElemCollection { PyElemCollection(self.0.uns.clone()) }

    #[setter(uns)]
    fn set_uns<'py>(&mut self, py: Python<'py>, mut uns: HashMap<String, &'py PyAny>) {
        let uns_ = uns.drain().map(|(k, v)| (k, to_rust_data1(py, v).unwrap())).collect();
        self.0.set_uns(&uns_).unwrap();
    }

    fn subset<'py>(
        &self,
        py: Python<'py>,
        obs_indices: Option<&'py PyAny>,
        var_indices: Option<&'py PyAny>,
    ) -> PyResult<()> {
        let n_obs = self.n_obs();
        let n_vars = self.n_vars();
        match obs_indices {
            Some(oidx) => {
                let i = to_indices(py, oidx, n_obs)?;
                match var_indices {
                    Some(vidx) => {
                        let j = to_indices(py, vidx, n_vars)?;
                        self.0.subset(i.as_slice(), j.as_slice());
                    },
                    None => self.0.subset_obs(i.as_slice()),
                }
            },
            None => {
               if let Some(vidx) = var_indices {
                    let j = to_indices(py, vidx, n_vars)?;
                    self.0.subset_var(j.as_slice());
               }
            },
        }
        Ok(())
    }
            
    #[getter]
    fn filename(&self) -> String { self.0.filename() }

    fn write(&self, filename: &str) {
        self.0.write(filename).unwrap();
    }

    fn import_mtx(&self, filename: &str, sorted: bool) {
        if utils::is_gzipped(filename) {
            let f = std::fs::File::open(filename).unwrap();
            let mut reader = std::io::BufReader::new(flate2::read::MultiGzDecoder::new(f));
            self.0.read_matrix_market(&mut reader, sorted).unwrap();
        } else {
            let f = std::fs::File::open(filename).unwrap();
            let mut reader = std::io::BufReader::new(f);
            self.0.read_matrix_market(&mut reader, sorted).unwrap();
        }
    }

    fn __repr__(&self) -> String {
        let mut descr = String::new();
        write!(
            &mut descr,
            "AnnData object with n_obs x n_vars = {} x {} backed at '{}'",
            self.n_obs(),
            self.n_vars(),
            self.filename(),
        ).unwrap();

        if let Some(obs) = self.get_obs() {
            write!(
                &mut descr,
                "\n    obs: {}",
                obs.0.get_column_names().unwrap().join(", "),
            ).unwrap();
        }
        if let Some(var) = self.get_var() {
            write!(
                &mut descr,
                "\n    var: {}",
                var.0.get_column_names().unwrap().join(", "),
            ).unwrap();
        }

        let obsm = self.get_obsm().keys();
        if obsm.len() > 0 {
            write!(&mut descr, "\n    obsm: {}", obsm.join(", ")).unwrap();
        }
        let obsp = self.get_obsp().keys();
        if obsp.len() > 0 {
            write!(&mut descr, "\n    obsp: {}", obsp.join(", ")).unwrap();
        }
        let varm = self.get_varm().keys();
        if varm.len() > 0 {
            write!(&mut descr, "\n    varm: {}", varm.join(", ")).unwrap();
        }
        let varp = self.get_varp().keys();
        if varp.len() > 0 {
            write!(&mut descr, "\n    varp: {}", varp.join(", ")).unwrap();
        }
        let uns = self.get_uns().keys();
        if uns.len() > 0 {
            write!(&mut descr, "\n    uns: {}", uns.join(", ")).unwrap();
        }
        descr
    }

    fn __str__(&self) -> String { self.__repr__() }
}

#[pyfunction]
pub fn read_h5ad(filename: &str, mode: &str) -> PyResult<AnnData> {
    let file = match mode {
        "r" => hdf5::File::open(filename).unwrap(),
        "r+" => hdf5::File::open_rw(filename).unwrap(),
        _ => panic!("Unkown mode"),
    };
    let anndata = base::AnnData::read(file).unwrap();
    Ok(AnnData(anndata))
}

#[pyfunction(sorted = "false")]
pub fn read_mtx<'py>(py: Python<'py>, input: &str, output: &str, sorted: bool) -> PyResult<AnnData> {
    let anndata = AnnData::new(py, output, None, None, None, None, None, None)?;
    anndata.import_mtx(input, sorted);
    Ok(anndata)
}

fn to_indices<'py>(py: Python<'py>, input: &'py PyAny, length: usize) -> PyResult<Vec<usize>> {
    if input.is_instance_of::<pyo3::types::PySlice>()? {
        let slice = input.downcast::<pyo3::types::PySlice>()?.indices(
            length.try_into().unwrap()
        )?;
        Ok(
            (slice.start.try_into().unwrap() ..= slice.stop.try_into().unwrap())
            .step_by(slice.step.try_into().unwrap()).collect()
        )
    } else if isinstance_of_arr(py, input)? {
        match input.getattr("dtype")?.getattr("name")?.extract::<&str>()? {
            "bool" => {
                let arr = input.extract::<numpy::PyReadonlyArrayDyn<bool>>()?.to_owned_array();
                let ndim = arr.ndim();
                let len = arr.len();
                if ndim == 1 && len == length {
                    Ok(
                        arr.into_iter().enumerate().filter_map(|(i, x)|
                            if x { Some(i) } else { None }).collect()
                    )
                } else {
                    panic!("dimension mismatched")
                }
            },
            _ => {
                let arr = input.extract::<numpy::PyReadonlyArrayDyn<i64>>()?.to_owned_array();
                let ndim = arr.ndim();
                if ndim == 1 {
                    Ok(arr.into_iter().map(|x| x.try_into().unwrap()).collect())
                } else {
                    panic!("dimension mismatched")
                }
            }
        }
    } else if input.is_instance_of::<pyo3::types::PyList>()? {
        Ok(input.extract::<Vec<i64>>()?.into_iter().map(|x| x.try_into().unwrap()).collect())
    } else {
        todo!()
    }
}