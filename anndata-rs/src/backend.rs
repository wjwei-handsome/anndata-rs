pub mod hdf5;

use crate::data::DynScalar;

use anyhow::{bail, Result};
use ndarray::{Array, Array2, ArrayView, Dimension};
use std::{ops::Deref, path::{Path, PathBuf}};
use core::fmt::{Display, Formatter};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DataType {
    Array(ScalarType),
    Categorical,
    CsrMatrix(ScalarType),
    CscMatrix(ScalarType),
    DataFrame,
    Scalar(ScalarType),
    Mapping,
}

impl Display for DataType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DataType::Array(t) => write!(f, "Array({})", t),
            DataType::Categorical => write!(f, "Categorical"),
            DataType::CsrMatrix(t) => write!(f, "CsrMatrix({})", t),
            DataType::CscMatrix(t) => write!(f, "CscMatrix({})", t),
            DataType::DataFrame => write!(f, "DataFrame"),
            DataType::Scalar(t) => write!(f, "Scalar({})", t),
            DataType::Mapping => write!(f, "Mapping"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ScalarType {
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    Bool,
    String,
}

impl Display for ScalarType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ScalarType::I8 => write!(f, "i8"),
            ScalarType::I16 => write!(f, "i16"),
            ScalarType::I32 => write!(f, "i32"),
            ScalarType::I64 => write!(f, "i64"),
            ScalarType::U8 => write!(f, "u8"),
            ScalarType::U16 => write!(f, "u16"),
            ScalarType::U32 => write!(f, "u32"),
            ScalarType::U64 => write!(f, "u64"),
            ScalarType::F32 => write!(f, "f32"),
            ScalarType::F64 => write!(f, "f64"),
            ScalarType::Bool => write!(f, "bool"),
            ScalarType::String => write!(f, "string"),
        }
    }
}   

/// A selection used for reading and writing to a Container.
pub enum Selection {
    All,
    Points(Array2<usize>),
}

pub enum DataContainer<B: Backend> {
    Group(B::Group),
    Dataset(B::Dataset),
}

impl<B: Backend> LocationOp for DataContainer<B> {
    type Backend = B;

    fn file(&self) -> Result<B::File> {
        match self {
            DataContainer::Group(g) => g.file(),
            DataContainer::Dataset(d) => d.file(),
        }
    }
    fn path(&self) -> PathBuf {
        match self {
            DataContainer::Group(g) => g.path(),
            DataContainer::Dataset(d) => d.path(),
        }
    }

    fn write_str_attr(&self, name: &str, value: &str) -> Result<()> {
        match self {
            DataContainer::Group(g) => g.write_str_attr(name, value),
            DataContainer::Dataset(d) => d.write_str_attr(name, value),
        }
    }
    fn write_str_arr_attr<'a, A, D>(&self, name: &str, value: A) -> Result<()>
    where
        A: Into<ArrayView<'a, String, D>>,
        D: ndarray::Dimension,
    {
        match self {
            DataContainer::Group(g) => g.write_str_arr_attr(name, value),
            DataContainer::Dataset(d) => d.write_str_arr_attr(name, value),
        }
    }

    fn read_str_attr(&self, name: &str) -> Result<String> {
        match self {
            DataContainer::Group(g) => g.read_str_attr(name),
            DataContainer::Dataset(d) => d.read_str_attr(name),
        }
    }
    fn read_str_arr_attr<D>(&self, name: &str) -> Result<Array<String, D>> {
        match self {
            DataContainer::Group(g) => g.read_str_arr_attr(name),
            DataContainer::Dataset(d) => d.read_str_arr_attr(name),
        }
    }
}

impl<B: Backend> DataContainer<B> {
    pub fn open(group: &B::Group, name: &str) -> Result<Self> {
        if group.exists(name)? {
            group.open_dataset(name)
                .map(DataContainer::Dataset)
                .or(group.open_group(name).map(DataContainer::Group))
        } else {
            bail!("No group or dataset named '{}' in group", name);
        }
    }

    pub fn delete(container: DataContainer<B>) -> Result<()> {
        let file = container.file()?;
        let name = container.path();
        let group = file.open_group(name.parent().unwrap().to_str().unwrap())?;

        group.delete(name.file_name().unwrap().to_str().unwrap())
    }

    pub fn encoding_type(&self) -> Result<DataType> {
        let enc = match self {
            DataContainer::Group(group) => {
                group.read_str_attr("encoding_type").unwrap_or("mapping".to_string())
            }
            DataContainer::Dataset(dataset) => {
                dataset.read_str_attr("encoding_type").unwrap_or("numeric-scalar".to_string())
            }
        };
        let ty = match enc.as_str() {
            "string" => DataType::Scalar(ScalarType::String),
            "numeric-scalar" => DataType::Scalar(self.as_dataset()?.dtype()?),
            "categorical" => DataType::Categorical,
            "string-array" => DataType::Array(ScalarType::String),
            "array" => DataType::Array(self.as_dataset()?.dtype()?),
            "csc_matrix" => todo!(),
            "csr_matrix" => {
                let ty = self.as_group()?.open_dataset("data")?.dtype()?;
                DataType::CsrMatrix(ty)
            },
            "dataframe" => DataType::DataFrame,
            "mapping" | "dict" => DataType::Mapping,
            ty => bail!("Unsupported type '{}'", ty),
        };
        Ok(ty)
    }

    pub fn as_group(&self) -> Result<&B::Group> {
        match self {
            Self::Group(x) => Ok(&x),
            _ => bail!("Expecting Group"),
        }
    }

    pub fn as_dataset(&self) -> Result<&B::Dataset> {
        match self {
            Self::Dataset(x) => Ok(&x),
            _ => bail!("Expecting Dataset"),
        }
    }
}

pub trait FileOp {
    type Backend: Backend;

    fn filename(&self) -> PathBuf;
    fn close(self) -> Result<()>;
}

pub trait GroupOp {
    type Backend: Backend;

    fn list(&self) -> Result<Vec<String>>;
    fn create_group(&self, name: &str) -> Result<<Self::Backend as Backend>::Group>;
    fn open_group(&self, name: &str) -> Result<<Self::Backend as Backend>::Group>;
    fn open_dataset(&self, name: &str) -> Result<<Self::Backend as Backend>::Dataset>; 
    fn delete(&self, name: &str) -> Result<()>;
    fn exists(&self, name: &str) -> Result<bool>;

    fn write_scalar<D: BackendData>(&self, name: &str, data: &D) -> Result<<Self::Backend as Backend>::Dataset>;
    fn write_array<'a, A, S, D, Dim>(
        &self,
        name: &str,
        data: A,
        selection: S,
    ) -> Result<<Self::Backend as Backend>::Dataset>
    where
        A: Into<ArrayView<'a, D, Dim>>,
        D: BackendData,
        S: Into<Selection>,
        Dim: Dimension;
}

pub trait LocationOp {
    type Backend: Backend;

    fn file(&self) -> Result<<Self::Backend as Backend>::File>;
    fn path(&self) -> PathBuf;

    fn write_str_attr(&self, name: &str, value: &str) -> Result<()>;
    fn write_str_arr_attr<'a, A, D>(&self, name: &str, value: A) -> Result<()>
    where
        A: Into<ArrayView<'a, String, D>>,
        D: ndarray::Dimension;

    fn read_str_attr(&self, name: &str) -> Result<String>;
    fn read_str_arr_attr<D>(&self, name: &str) -> Result<Array<String, D>>;
}

pub trait DatasetOp {
    type Backend: Backend;

    fn dtype(&self) -> Result<ScalarType>;
    fn shape(&self) -> Result<Vec<usize>>;

    fn read_scalar<T: BackendData>(&self) -> Result<T>;

    fn read_array<T: BackendData, S, D>(
        &self,
        selection: S,
    ) -> Result<Array<T, D>>
    where
        S: Into<Selection>;
}

pub trait Backend {
    type File: FileOp<Backend = Self> + GroupOp<Backend = Self>;

    /// Groups work like dictionaries.
    type Group: GroupOp<Backend = Self> + LocationOp<Backend=Self>;

    /// datasets contain arrays.
    type Dataset: DatasetOp<Backend = Self> + LocationOp<Backend=Self>;

    fn create<P: AsRef<Path>>(path: P) -> Result<Self::File>;
}

pub enum DynArrayView<'a, D> {
    I8(ArrayView<'a, i8, D>),
    I16(ArrayView<'a, i16, D>),
    I32(ArrayView<'a, i32, D>),
    I64(ArrayView<'a, i64, D>),
    U8(ArrayView<'a, u8, D>),
    U16(ArrayView<'a, u16, D>),
    U32(ArrayView<'a, u32, D>),
    U64(ArrayView<'a, u64, D>),
    F32(ArrayView<'a, f32, D>),
    F64(ArrayView<'a, f64, D>),
    String(ArrayView<'a, String, D>),
    Bool(ArrayView<'a, bool, D>),
}

pub trait BackendData: Send + Sync + Clone + 'static {
    const DTYPE: ScalarType;
    fn into_dyn(&self) -> DynScalar;
    fn into_dyn_arr<'a, D>(arr: ArrayView<'a, Self, D>) -> DynArrayView<'a, D>;
    fn from_dyn(x: DynScalar) -> Result<Self>;
}

impl BackendData for i8 {
    const DTYPE: ScalarType = ScalarType::I8;

    fn into_dyn(&self) -> DynScalar {
        DynScalar::I8(*self)
    }

    fn into_dyn_arr<'a, D>(arr: ArrayView<'a, Self, D>) -> DynArrayView<'a, D> {
        DynArrayView::I8(arr)
    }

    fn from_dyn(x: DynScalar) -> Result<Self> {
        if let DynScalar::I8(x) = x {
            Ok(x)
        } else {
            bail!("Expecting i8")
        }
    }
}

impl BackendData for i16 {
    const DTYPE: ScalarType = ScalarType::I16;

    fn into_dyn(&self) -> DynScalar {
        DynScalar::I16(*self)
    }

    fn into_dyn_arr<'a, D>(arr: ArrayView<'a, Self, D>) -> DynArrayView<'a, D> {
        DynArrayView::I16(arr)
    }

    fn from_dyn(x: DynScalar) -> Result<Self> {
        if let DynScalar::I16(x) = x {
            Ok(x)
        } else {
            bail!("Expecting i16")
        }
    }
}

impl BackendData for i32 {
    const DTYPE: ScalarType = ScalarType::I32;

    fn into_dyn(&self) -> DynScalar {
        DynScalar::I32(*self)
    }

    fn into_dyn_arr<'a, D>(arr: ArrayView<'a, Self, D>) -> DynArrayView<'a, D> {
        DynArrayView::I32(arr)
    }

    fn from_dyn(x: DynScalar) -> Result<Self> {
        if let DynScalar::I32(x) = x {
            Ok(x)
        } else {
            bail!("Expecting i32")
        }
    }
}

impl BackendData for i64 {
    const DTYPE: ScalarType = ScalarType::I64;

    fn into_dyn(&self) -> DynScalar {
        DynScalar::I64(*self)
    }

    fn into_dyn_arr<'a, D>(arr: ArrayView<'a, Self, D>) -> DynArrayView<'a, D> {
        DynArrayView::I64(arr)
    }

    fn from_dyn(x: DynScalar) -> Result<Self> {
        if let DynScalar::I64(x) = x {
            Ok(x)
        } else {
            bail!("Expecting i64")
        }
    }
}

impl BackendData for u8 {
    const DTYPE: ScalarType = ScalarType::U8;

    fn into_dyn(&self) -> DynScalar {
        DynScalar::U8(*self)
    }

    fn into_dyn_arr<'a, D>(arr: ArrayView<'a, Self, D>) -> DynArrayView<'a, D> {
        DynArrayView::U8(arr)
    }

    fn from_dyn(x: DynScalar) -> Result<Self> {
        if let DynScalar::U8(x) = x {
            Ok(x)
        } else {
            bail!("Expecting u8")
        }
    }
}

impl BackendData for u16 {
    const DTYPE: ScalarType = ScalarType::U16;

    fn into_dyn(&self) -> DynScalar {
        DynScalar::U16(*self)
    }

    fn into_dyn_arr<'a, D>(arr: ArrayView<'a, Self, D>) -> DynArrayView<'a, D> {
        DynArrayView::U16(arr)
    }

    fn from_dyn(x: DynScalar) -> Result<Self> {
        if let DynScalar::U16(x) = x {
            Ok(x)
        } else {
            bail!("Expecting u16")
        }
    }
}

impl BackendData for u32 {
    const DTYPE: ScalarType = ScalarType::U32;

    fn into_dyn(&self) -> DynScalar {
        DynScalar::U32(*self)
    }

    fn into_dyn_arr<'a, D>(arr: ArrayView<'a, Self, D>) -> DynArrayView<'a, D> {
        DynArrayView::U32(arr)
    }

    fn from_dyn(x: DynScalar) -> Result<Self> {
        if let DynScalar::U32(x) = x {
            Ok(x)
        } else {
            bail!("Expecting u32")
        }
    }
}

impl BackendData for u64 {
    const DTYPE: ScalarType = ScalarType::U64;

    fn into_dyn(&self) -> DynScalar {
        DynScalar::U64(*self)
    }
    
    fn into_dyn_arr<'a, D>(arr: ArrayView<'a, Self, D>) -> DynArrayView<'a, D> {
        DynArrayView::U64(arr)
    }

    fn from_dyn(x: DynScalar) -> Result<Self> {
        if let DynScalar::U64(x) = x {
            Ok(x)
        } else {
            bail!("Expecting u64")
        }
    }
}

impl BackendData for f32 {
    const DTYPE: ScalarType = ScalarType::F32;

    fn into_dyn(&self) -> DynScalar {
        DynScalar::F32(*self)
    }

    fn into_dyn_arr<'a, D>(arr: ArrayView<'a, Self, D>) -> DynArrayView<'a, D> {
        DynArrayView::F32(arr)
    }

    fn from_dyn(x: DynScalar) -> Result<Self> {
        if let DynScalar::F32(x) = x {
            Ok(x)
        } else {
            bail!("Expecting f32")
        }
    }
}

impl BackendData for f64 {
    const DTYPE: ScalarType = ScalarType::F64;

    fn into_dyn(&self) -> DynScalar {
        DynScalar::F64(*self)
    }

    fn into_dyn_arr<'a, D>(arr: ArrayView<'a, Self, D>) -> DynArrayView<'a, D> {
        DynArrayView::F64(arr)
    }

    fn from_dyn(x: DynScalar) -> Result<Self> {
        if let DynScalar::F64(x) = x {
            Ok(x)
        } else {
            bail!("Expecting f64")
        }
    }
}

impl BackendData for String {
    const DTYPE: ScalarType = ScalarType::String;

    fn into_dyn(&self) -> DynScalar {
        DynScalar::String(self.clone())
    }

    fn into_dyn_arr<'a, D>(arr: ArrayView<'a, Self, D>) -> DynArrayView<'a, D> {
        DynArrayView::String(arr)
    }

    fn from_dyn(x: DynScalar) -> Result<Self> {
        if let DynScalar::String(x) = x {
            Ok(x)
        } else {
            bail!("Expecting string")
        }
    }
}

impl BackendData for bool {
    const DTYPE: ScalarType = ScalarType::Bool;

    fn into_dyn(&self) -> DynScalar {
        DynScalar::Bool(*self)
    }

    fn into_dyn_arr<'a, D>(arr: ArrayView<'a, Self, D>) -> DynArrayView<'a, D> {
        DynArrayView::Bool(arr)
    }

    fn from_dyn(x: DynScalar) -> Result<Self> {
        if let DynScalar::Bool(x) = x {
            Ok(x)
        } else {
            bail!("Expecting bool")
        }
    }
}

pub fn iter_containers<B: Backend>(group: &B::Group) -> impl Iterator<Item = (String, DataContainer<B>)> + '_{
    group.list().unwrap().into_iter().map(|x| {
        let container = DataContainer::open(group, &x).unwrap();
        (x, container)
    })
}