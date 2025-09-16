import lustre/attribute as a
import lustre/element/html as h

pub fn create_email_field(field_name: String, label: String) {
  h.input([
    a.class(
      "block w-full px-4 py-2 mt-2 text-gray-700 placeholder-gray-500 bg-white border rounded-lg dark:bg-gray-800 dark:border-gray-600 dark:placeholder-gray-400 focus:border-blue-400 dark:focus:border-blue-300 focus:ring-opacity-40 focus:outline-none focus:ring focus:ring-blue-300",
    ),
    a.type_("email"),
    a.name(field_name),
    a.placeholder(label),
    a.attribute("aria-label", label),
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
