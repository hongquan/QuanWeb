import formal/form.{type Form} as formlib
import lustre/attribute as a
import lustre/element/html as h
import lustre/event as ev
import views/ui_components.{render_flash_messages}

import core.{
  type FlashMessage, type LoadingStatus, type LoginData, IsSubmitting,
  UserSubmittedLoginForm,
}
import views/widgets

pub fn make_login_page(
  loading_status: LoadingStatus,
  form: Form(LoginData),
  flash_messages: List(FlashMessage),
) {
  let handle_submit = fn(values) {
    form |> formlib.add_values(values) |> formlib.run |> UserSubmittedLoginForm
  }
  h.div(
    [
      a.class(
        "w-full max-w-md mx-auto mt-16 overflow-hidden bg-white rounded-lg shadow-md dark:bg-gray-800",
      ),
    ],
    [
      render_flash_messages(flash_messages),
      h.div([a.class("max-w-sm mx-auto px-6 py-4")], [
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
            widgets.auto_submit_button(
              core.Blue,
              "Login",
              loading_status == IsSubmitting,
            ),
          ]),
        ]),
      ]),
    ],
  )
}
