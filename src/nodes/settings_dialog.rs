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

use gdnative::{api::WindowDialog, methods, prelude::*, NativeClass};

// TODO: Use window node for 4.0
#[derive(NativeClass)]
#[inherit(WindowDialog)]
pub struct SettingsDialog;

#[methods]
impl SettingsDialog {
    fn new(_owner: &WindowDialog) -> Self {
        SettingsDialog
    }

    #[export]
    fn _ready(&self, owner: TRef<WindowDialog>) {
        owner.set_title("Open2DHolo Settings");
    }
}
