import gleam/javascript/array.{type Array as JSArray}
import plinth/browser/element.{type Element}

@external(javascript, "./element.ffi.mjs", "getUpFormData")
pub fn get_up_form_data(select: Element) -> JSArray(#(String, String))

@external(javascript, "./element.ffi.mjs", "getFormFieldValue")
pub fn get_form_field_value(
  select: Element,
  name: String,
) -> Result(String, Nil)

@external(javascript, "./element.ffi.mjs", "showDialog")
pub fn show_dialog(selector: String) -> Bool

@external(javascript, "./element.ffi.mjs", "submitForm")
pub fn submit_form(form: Element) -> Nil

@external(javascript, "./element.ffi.mjs", "getFormData")
pub fn get_form_data(form: Element) -> JSArray(#(String, String))
