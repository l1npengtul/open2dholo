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
