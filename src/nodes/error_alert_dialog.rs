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

use gdnative::{
    api::{AcceptDialog, Tree, TreeItem},
    methods,
    prelude::*,
    NativeClass,
};

// TODO: Use window node for 4.0
#[derive(NativeClass)]
#[inherit(AcceptDialog)]
pub struct ErrorAlertDialog;

#[methods]
impl ErrorAlertDialog {
    // register the ErrorDialogShow signal
    fn register_signals(builder: &ClassBuilder<Self>) {
        builder.add_signal(Signal {
            name: "error_occour_default",
            args: &[
                SignalArgument {
                    name: "error_name",
                    default: Variant::from_str("GenericError"),
                    export_info: ExportInfo::new(VariantType::GodotString),
                    usage: PropertyUsage::DEFAULT,
                },
                SignalArgument {
                    name: "error_desc",
                    default: Variant::from_str("i love emilia"),
                    export_info: ExportInfo::new(VariantType::GodotString),
                    usage: PropertyUsage::DEFAULT,
                },
            ],
        });
        builder.add_signal(Signal {
            name: "error_occour_with_stacktrace",
            args: &[
                SignalArgument {
                    name: "error_name",
                    default: Variant::from_str("GenericError"),
                    export_info: ExportInfo::new(VariantType::GodotString),
                    usage: PropertyUsage::DEFAULT,
                },
                SignalArgument {
                    name: "error_desc",
                    default: Variant::from_str("i love emilia"),
                    export_info: ExportInfo::new(VariantType::GodotString),
                    usage: PropertyUsage::DEFAULT,
                },
                SignalArgument {
                    name: "error_stack",
                    default: Variant::from_str("main.rs 1: i love emilia"),
                    export_info: ExportInfo::new(VariantType::GodotString),
                    usage: PropertyUsage::DEFAULT,
                },
            ],
        });
    }

    fn new(_owner: &ErrorAlertDialog) -> Self {
        ErrorAlertDialog
    }

    #[export]
    fn _ready(&self, owner: TRef<AcceptDialog>) {
        owner.set_title("Error");
    }

    #[export]
    pub fn on_error_occour_with_stacktrace(
        error_name: Variant,
        error_desc: Variant,
        error_stack: Variant,
    ) {
    }

    #[export]
    pub fn on_error_occour_default(error_name: Variant, error_desc: Variant) {}
}
