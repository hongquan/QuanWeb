import gleam/javascript/array.{type Array as JSArray}
import plinth/browser/element.{type Element}

@external(javascript, "./element.ffi.mjs", "getFormData")
pub fn get_form_data(select: Element) -> JSArray(#(String, String))
