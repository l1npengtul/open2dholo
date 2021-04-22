//     Open2DHolo - Open 2D Holo, a program to procedurally animate your face onto an 3D Model.
//     Copyright (C) 2020-2021 l1npengtul
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

#[macro_export]
macro_rules! handle_boxerr {
    ($a:expr) => {{
        if let Err(why) = $a {
            return Err(why);
        }
    }};
    ($a:expr, $b:expr) => {{
        if let Err(_) = $a {
            return $b;
        }
    }};
}

#[macro_export]
macro_rules! show_error {
    ($err_name:expr, $err_desc:expr) => {{
        let os = gdnative::api::OS::godot_singleton();
        os.emit_signal(
            "error_occur_default",
            &[
                Variant::from_str(format!("{}", $err_name)),
                Variant::from_str(format!("{}", $err_desc)),
            ],
        )
    }};
}

#[macro_export]
macro_rules! wtf {
    ($result:expr) => {{
        match $result {
            Ok(a) => a,
            Err(why) => {
                let file: &'static str = std::file!();
                let line: u32 = std::line!();
                let cols: u32 = std::column!();
                let why_str = format!("{}", why);
                let os: &'static gdnative::api::OS = gdnative::api::OS::godot_singleton();
                os.alert(
                    format!(
                        "Fatal Error at file {}, {}:{}. \nResult message: {}",
                        file, line, cols, why_str
                    ),
                    format!("Open2DHolo Fatal Error"),
                );
                // get scene tree
                // os.emit_signal("error_critical", &[Variant::from_i64(1)]);
                std::process::exit(1);
            }
        }
    }};
    ($result:expr, $reason:expr) => {{
        match $result {
            Ok(a) => a,
            Err(why) => {
                let file: &'static str = std::file!();
                let line: u32 = std::line!();
                let cols: u32 = std::column!();
                let why_str = format!("{}", $reason);
                let os: &'static gdnative::api::OS = gdnative::api::OS::godot_singleton();
                os.alert(
                    format!(
                        "Fatal Error at file {}, {}:{}. \nResult message: {}",
                        file, line, cols, why_str
                    ),
                    format!("Open2DHolo Fatal Error"),
                );
                // get scene tree
                // os.emit_signal("error_critical", &[Variant::from_i64(1)]);
                std::process::exit(1);
            }
        }
    }};
}

#[macro_export]
macro_rules! globalize_path {
    ($path:expr) => {{
        let proj: &'static gdnative::api::ProjectSettings =
            gdnative::api::ProjectSettings::godot_singleton();
        format!("{}", proj.globalize_path($path))
    }};
}

#[macro_export]
macro_rules! localize_path {
    ($path:expr) => {{
        let proj: &'static gdnative::api::ProjectSettings =
            gdnative::api::ProjectSettings::godot_singleton();
        format!("{}", proj.localize_path($path))
    }};
}
