//     Open2DH - Open 2D Holo, a program to procedurally animate your face onto an 3D Model.
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

//
// macro_rules! error_handle_string {
//     (&string: expr) => {
//     //     call function
//     }
// }
//
use gdnative::api::OS;

#[macro_export]
macro_rules! make_dyn {
    ($a:expr,$b:path) => {{
        let err: Box<dyn $b> = Box::new($a);
        Err(err)
    }};
    ($a:expr) => {{
        let err: Box<dyn std::error::Error> = Box::new($a);
        Err(err)
    }};
}

#[macro_export]
macro_rules! ret_boxerr {
    ($a:expr) => {{
        let err: Box<dyn std::error::Error> = Box::new($a);
        return Err(err);
    }};
}
