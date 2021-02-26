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
}

#[macro_export]
macro_rules! lock_gil {
    () => {
        let gil = Python::acquire_gil();
        let py = gil.python();
    };
}

#[macro_export]
macro_rules! make_pymod {
    ($py:expr,$pyany:expr) => {{
        let pyany_ref: &PyModule = $pyany.as_ref($py);
        pyany_ref
    }};
    ($pyany:expr) => {{
        let gil: GILGuard = Python::acquire_gil();
        let pyany_ref: &PyModule = $pyany.as_ref(gil.python());
        pyany_ref
    }};
}
