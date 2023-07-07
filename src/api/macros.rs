
#[macro_export]
macro_rules! append_set_statement {
    ($name:literal, $etype:literal, $line_vec:ident, $name_list:ident) => {
        if $name_list.iter().any(|&f| f == $name) {
            $line_vec.push(concat!($name, " := <", $etype, "> $", $name));
        }
    };
}

pub use append_set_statement;
