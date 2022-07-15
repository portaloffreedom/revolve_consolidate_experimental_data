use regex::Regex;
use crate::error::{ConvertError, ConvertResult, Error};

pub struct Vector2<F> {
    pub x: F,
    pub y: F,
}

pub struct Vector3<F> {
    pub x: F,
    pub y: F,
    pub z: F,
}

impl<F> Vector2<F> {
    pub fn new(x: F, y: F) -> Self {
        Self {x, y}
    }

    pub fn parse_from_python<S>(data: S) -> Result<Self, Error>
    where
        S: AsRef<str>,
        F: std::str::FromStr,
        F::Err: 'static + std::error::Error,
    {
        lazy_static! {
                static ref VECTOR2_PYTHON_REGEX: Regex =
                    Regex::new(r"^\((-?\d+), (-?\d+)\)$").unwrap();
            }
        let captured = VECTOR2_PYTHON_REGEX.captures(data.as_ref())
            .ok_or(Error::new("Failed to parse the python vector"))?;

        let x = captured[1].parse::<F>().into_error("can't parse x")?;
        let y = captured[2].parse::<F>().into_error("can't parse y")?;
        Ok(Self { x, y })
    }
}

impl<F> Vector3<F> {
    pub fn new(x: F, y: F, z: F) -> Self {
        Self {x, y, z}
    }

    pub fn parse_from_python<S>(data: S) -> Result<Self, Error>
        where
            S: AsRef<str>,
            F: std::str::FromStr,
            F::Err: 'static + std::error::Error,
    {
        lazy_static! {
                static ref VECTOR3_PYTHON_REGEX: Regex =
                    Regex::new(r"^\((-?\d+), (-?\d+), (-?\d+)\)$").unwrap();
            }
        let captured = VECTOR3_PYTHON_REGEX.captures(data.as_ref())
            .ok_or(Error::new("Failed to parse the python vector"))?;

        let x = captured[1].parse::<F>().into_error("can't parse x")?;
        let y = captured[2].parse::<F>().into_error("can't parse y")?;
        let z = captured[3].parse::<F>().into_error("can't parse z")?;
        Ok(Self { x, y, z })
    }
}