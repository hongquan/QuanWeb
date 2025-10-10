import formal/form.{type Form} as formlib
import lustre/attribute as a
import lustre/element/html as h
import lustre/event as ev

import core.{type LoginData, UserSubmittedLoginForm}
import views/widgets

pub fn make_login_page(form: Form(LoginData)) {
  let handle_submit = fn(values) {
    form |> formlib.add_values(values) |> formlib.run |> UserSubmittedLoginForm
  }
  h.div(
    [
      a.class(
        "w-full max-w-sm mx-auto mt-16 overflow-hidden bg-white rounded-lg shadow-md dark:bg-gray-800",
      ),
    ],
    [
      h.div([a.class("px-6 py-4")], [
        h.h3(
          [
            a.class(
              "mt-3 text-xl font-medium text-center text-gray-600 dark:text-gray-200",
            ),
          ],
          [h.text("Login")],
        ),
        h.form([a.method("post"), ev.on_submit(handle_submit)], [
          h.div([a.class("w-full mt-4")], [
            widgets.create_email_field("email", "Email Address", True),
          ]),
          h.div([a.class("w-full mt-4")], [
            widgets.create_password_field("password", "Password"),
          ]),
          h.div([a.class("flex items-center justify-between mt-4")], [
            h.a(
              [
                a.class(
                  "text-sm text-gray-600 dark:text-gray-200 hover:text-gray-500",
                ),
                a.href("#"),
              ],
              [h.text("Forget Password?")],
            ),
            h.button(
              [
                a.class(
                  "px-6 py-2 text-sm font-medium tracking-wide text-white capitalize transition-colors duration-300 transform bg-blue-500 rounded-lg hover:bg-blue-400 focus:outline-none focus:ring focus:ring-blue-300 focus:ring-opacity-50 cursor-pointer",
                ),
                a.type_("submit"),
              ],
              [h.text("Sign In")],
            ),
          ]),
        ]),
      ]),
    ],
  )
}
