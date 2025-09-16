import formal/form as formlib
import lustre/attribute as a
import lustre/element/html as h
import lustre/event as ev

import core.{type LoginData, UserSubmittedLoginForm}

pub fn make_login_page(form: formlib.Form(LoginData)) {
  let handle_submit = fn(values) {
    form |> formlib.add_values(values) |> formlib.run |> UserSubmittedLoginForm
  }
  h.div(
    [
      a.class(
        "w-full max-w-sm mx-auto overflow-hidden bg-white rounded-lg shadow-md dark:bg-gray-800",
      ),
    ],
    [
      h.div([a.class("px-6 py-4")], [
        h.div([a.class("flex justify-center mx-auto")], [
          h.img([
            a.class("w-auto h-7 sm:h-8"),
            a.src("https://merakiui.com/images/logo.svg"),
            a.alt(""),
          ]),
        ]),
        h.h3(
          [
            a.class(
              "mt-3 text-xl font-medium text-center text-gray-600 dark:text-gray-200",
            ),
          ],
          [h.text("Welcome Back")],
        ),
        h.p([a.class("mt-1 text-center text-gray-500 dark:text-gray-400")], [
          h.text("Login or create account"),
        ]),
        h.form([a.method("post"), ev.on_submit(handle_submit)], [
          h.div([a.class("w-full mt-4")], [
            h.input([
              a.class(
                "block w-full px-4 py-2 mt-2 text-gray-700 placeholder-gray-500 bg-white border rounded-lg dark:bg-gray-800 dark:border-gray-600 dark:placeholder-gray-400 focus:border-blue-400 dark:focus:border-blue-300 focus:ring-opacity-40 focus:outline-none focus:ring focus:ring-blue-300",
              ),
              a.type_("email"),
              a.name("email"),
              a.placeholder("Email Address"),
              a.attribute("aria-label", "Email Address"),
            ]),
          ]),
          h.div([a.class("w-full mt-4")], [
            h.input([
              a.class(
                "block w-full px-4 py-2 mt-2 text-gray-700 placeholder-gray-500 bg-white border rounded-lg dark:bg-gray-800 dark:border-gray-600 dark:placeholder-gray-400 focus:border-blue-400 dark:focus:border-blue-300 focus:ring-opacity-40 focus:outline-none focus:ring focus:ring-blue-300",
              ),
              a.type_("password"),
              a.name("password"),
              a.placeholder("Password"),
              a.attribute("aria-label", "Password"),
            ]),
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
      h.div(
        [
          a.class(
            "flex items-center justify-center py-4 text-center bg-gray-50 dark:bg-gray-700",
          ),
        ],
        [
          h.span([a.class("text-sm text-gray-600 dark:text-gray-200")], [
            h.text("Don't have an account? "),
          ]),
          h.a(
            [
              a.class(
                "mx-2 text-sm font-bold text-blue-500 dark:text-blue-400 hover:underline",
              ),
              a.href("#"),
            ],
            [h.text("Register")],
          ),
        ],
      ),
    ],
  )
}
