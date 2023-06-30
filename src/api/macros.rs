#[macro_export]
macro_rules! append_field_general {
    ($name:literal, $card:expr, $elm_vec:ident, $val_vec:ident, $field:expr, $name_list:ident) => {
        if $name_list.iter().any(|&f| f == $name) {
            $val_vec.push($field.clone().map(EValue::from));
            $elm_vec.push(create_shape_element($name, $card));
        }
    };
}

#[macro_export]
macro_rules! append_field {
    ($name:literal, $etype:expr, $card:expr, $elm_vec:ident, $val_vec:ident, $field:expr, $name_list:ident) => {
        if $name_list.iter().any(|&f| f == $name) {
            $val_vec.push($field.clone().map($etype));
            $elm_vec.push(create_shape_element($name, $card));
        }
    };
}

#[macro_export]
macro_rules! append_set_statement {
    ($name:literal, $etype:literal, $line_vec:ident, $name_list:ident) => {
        if $name_list.iter().any(|&f| f == $name) {
            $line_vec.push(concat!($name, " := <", $etype, "> $", $name));
        }
    };
}

pub use append_field_general;
pub use append_field;
pub use append_set_statement;
