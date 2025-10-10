import gleam/list
import gleam/option.{type Option, Some}
import lustre/attribute as a
import lustre/element/html as h
import views/load_indicators

pub fn create_email_field(field_name: String, label: String, autofocus: Bool) {
  h.input([
    a.class(
      "block w-full px-4 py-2 mt-2 text-gray-700 placeholder-gray-500 bg-white border rounded-lg dark:bg-gray-800 dark:border-gray-600 dark:placeholder-gray-400 focus:border-blue-400 dark:focus:border-blue-300 focus:ring-opacity-40 focus:outline-none focus:ring focus:ring-blue-300",
    ),
    a.type_("email"),
    a.name(field_name),
    a.placeholder(label),
    a.attribute("aria-label", label),
    a.autofocus(autofocus),
  ])
}

pub fn create_password_field(field_name: String, label: String) {
  h.input([
    a.class(
      "block w-full px-4 py-2 mt-2 text-gray-700 placeholder-gray-500 bg-white border rounded-lg dark:bg-gray-800 dark:border-gray-600 dark:placeholder-gray-400 focus:border-blue-400 dark:focus:border-blue-300 focus:ring-opacity-40 focus:outline-none focus:ring focus:ring-blue-300",
    ),
    a.type_("password"),
    a.name(field_name),
    a.placeholder(label),
    a.attribute("aria-label", label),
  ])
}

pub fn render_single_select(
  field_name: String,
  choices: List(#(String, String)),
  selected_value: Option(String),
  title: Option(String),
) {
  let options =
    choices
    |> list.map(fn(kv) {
      let #(value, name) = kv
      h.option(
        [
          a.value(value),
          a.selected(
            selected_value
            |> option.map(fn(x) { x == value })
            |> option.unwrap(False),
          ),
        ],
        name,
      )
    })
  let options = case title {
    Some(title) -> [h.option([a.disabled(True)], title), ..options]
    _ -> options
  }
  h.select(
    [
      a.name(field_name),
      a.class("border dark:border-gray-600 rounded-md py-2 ps-2 pe-4"),
    ],
    options,
  )
}

pub fn submit_button(is_submitting: Bool) {
  let #(class_button, children) = case is_submitting {
    True -> #("px-4 flex flex-row items-center space-x-2", [
      load_indicators.wifi_signal("h-5 w-5 fill-current"),
      h.span([], [h.text("Sign In")]),
    ])
    False -> #("px-6", [h.text("Sign In")])
  }

  h.button(
    [
      a.class(
        "py-2 text-sm font-medium tracking-wide text-white capitalize transition-colors duration-300 transform bg-blue-500 rounded-lg hover:bg-blue-400 focus:outline-none focus:ring focus:ring-blue-300 focus:ring-opacity-50 cursor-pointer",
      ),
      a.class(class_button),
      a.type_("submit"),
    ],
    children,
  )
}
