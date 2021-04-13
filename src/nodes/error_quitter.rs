use gdnative::{api::Control, methods, prelude::*, NativeClass};
#[derive(NativeClass)]
#[inherit(Control)]
#[register_with(Self::register_signals)]
pub struct ErrorQuitter;

#[methods]
impl ErrorQuitter {
    fn register_signals(builder: &ClassBuilder<Self>) {
        builder.add_signal(Signal {
            name: "error_critical",
            args: &[SignalArgument {
                name: "error_code",
                default: Variant::from_i64(1),
                export_info: ExportInfo::new(VariantType::I64),
                usage: PropertyUsage::DEFAULT,
            }],
        });
    }

    pub fn new(_owner: &Control) -> Self {
        ErrorQuitter
    }

    #[export]
    pub fn _ready(&self, owner: TRef<Control>) {
        if let Err(_why) = {
            owner.connect(
                "error_critical",
                owner,
                "quit_application",
                VariantArray::new_shared(),
                0,
            )
        } {
            std::process::abort(); // let the OS take care of destructors. I'm out.
        }
    }

    #[export]
    pub fn quit_application(&self, owner: TRef<Control>, error_code: Variant) {
        match owner.get_tree() {
            Some(tree) => {
                let error = i64::from_variant(&error_code).unwrap_or(-1);
                let scenetree = unsafe { tree.assume_safe() };
                scenetree.quit(error)
            }
            None => {
                // panic!("Why am I orphaned from the scenetree? This is (probably) not a Open2DHolo bug!");
                std::process::abort(); // let the OS take care of destructors. I'm out.
            }
        };
    }
}
