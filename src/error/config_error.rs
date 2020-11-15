//     Copyright (C) 2020-2021l1npengtul
//
//     This program is free software: you can redistribute it and/or modify
//     it under the terms of the GNU General Public License as published by
//     the Free Software Foundation, either version 3 of the License, or
//     (at your option) any later version.
//
//     This program is distributed in the hope that it will be useful,
//     but WITHOUT ANY WARRANTY; without even the implied warranty of
//     MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//     GNU General Public License for more details.
//
//     You should have received a copy of the GNU General Public License
//     along with this program.  If not, see <https://www.gnu.org/licenses/>.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Config file not found at location: {0}!")]
    FileNotFound(String),
    #[error("Config file at {0} is invalid!")]
    InvalidConfiguration(String),
    #[error("Path is invalid! (Could not be converted)")]
    InvalidPath,
    #[error("General Error, Could not load config.")]
    General,
}
