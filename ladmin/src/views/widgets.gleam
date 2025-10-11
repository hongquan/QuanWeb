import core.{type Color}
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

fn submit_button(
  is_auto: Bool,
  color: Color,
  label: String,
  click_handler: a.Attribute(core.Msg(a)),
  is_submitting: Bool,
) {
  let #(class_button, children) = case is_submitting {
    True -> #("px-4 flex flex-row items-center space-x-2", [
      load_indicators.wifi_signal("h-5 w-5 fill-current"),
      h.span([], [h.text(label)]),
    ])
    False -> #("px-6", [h.text(label)])
  }
  let button_type = case is_auto {
    True -> a.type_("submit")
    False -> a.type_("button")
  }
  let class_color = case color {
    core.Blue -> "bg-blue-500 hover:bg-blue-400 focus:ring-blue-300"
    core.Sky -> "bg-sky-700 hover:bg-sky-600 focus:bg-sky-500 "
    core.Purple -> "bg-purple-700 hover:bg-purple-600 focus:bg-purple-500"
  }
  h.button(
    [
      a.class(
        "py-2 text-sm font-medium tracking-wide text-white capitalize transition-colors duration-300 transform rounded-lg focus:outline-none focus:ring focus:ring-opacity-50 cursor-pointer",
      ),
      a.class(class_color),
      a.class(class_button),
      button_type,
      click_handler,
    ],
    children,
  )
}

/// Button of "submit" type (to make form submit by just pressing Enter anywhere)
pub fn auto_submit_button(color: Color, label: String, is_submitting: Bool) {
  submit_button(True, color, label, a.none(), is_submitting)
}

/// Button of "button" type (to not make form auto submit)
pub fn manual_submit_button(
  color: Color,
  label: String,
  click_handler: a.Attribute(core.Msg(a)),
  is_submitting: Bool,
) {
  submit_button(False, color, label, click_handler, is_submitting)
}
