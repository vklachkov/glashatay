// Эти предупреждения конфликтуют с API валидатора Garde.
#![allow(clippy::trivially_copy_pass_by_ref, clippy::ptr_arg)]
// Валидаторы взяты из другого проекта и могут пригодится позже.
#![allow(unused)]

use crate::utils::PrintErrorChain;
use std::time::Duration;
use std::{fs, io, path::PathBuf};
use url::Url;

pub fn is_base_url(url: &Url, _: &()) -> garde::Result {
    if url.cannot_be_a_base() {
        Err(garde::Error::new(
            "url should be base, e.g. https://site.com/",
        ))
    } else {
        Ok(())
    }
}

pub fn non_zero_duration(duration: &Duration, _: &()) -> garde::Result {
    if *duration == Duration::ZERO {
        Err(garde::Error::new("value can not be zero"))
    } else {
        Ok(())
    }
}

pub fn is_file_and_exists(path: &PathBuf, _: &()) -> garde::Result {
    let dpath = path.display();

    let is_exists = path.try_exists().map_err(IoValidationError)?;
    if !is_exists {
        return Err(garde::Error::new(format!("path '{dpath}' does not exists")));
    }

    let metadata = fs::metadata(path).map_err(IoValidationError)?;
    if !metadata.is_file() {
        return Err(garde::Error::new(format!("'{dpath}' is not a file")));
    }

    Ok(())
}

pub fn is_file_directory_exists(path: &PathBuf, _: &()) -> garde::Result {
    let Some(path) = path.parent() else {
        return Ok(());
    };

    let dpath = path.display();

    let is_exists = path.try_exists().map_err(IoValidationError)?;
    if !is_exists {
        return Err(garde::Error::new(format!("path '{dpath}' does not exists")));
    }

    let metadata = fs::metadata(path).map_err(IoValidationError)?;
    if !metadata.is_dir() {
        return Err(garde::Error::new(format!("'{dpath}' is not a file")));
    }

    Ok(())
}

pub fn is_directory_and_exists(path: &PathBuf, _: &()) -> garde::Result {
    let dpath = path.display();

    let is_exists = path.try_exists().map_err(IoValidationError)?;
    if !is_exists {
        return Err(garde::Error::new(format!("path '{dpath}' does not exists")));
    }

    let metadata = fs::metadata(path).map_err(IoValidationError)?;
    if !metadata.is_dir() {
        return Err(garde::Error::new(format!("'{dpath}' is not a directory")));
    }

    Ok(())
}

struct IoValidationError(io::Error);

impl From<IoValidationError> for garde::Error {
    fn from(error: IoValidationError) -> Self {
        garde::Error::new(format!(
            "io error during validation: {err}",
            err = PrintErrorChain(&error.0)
        ))
    }
}
