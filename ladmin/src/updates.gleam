import formal/form as formlib
import gleam/dynamic/decode
import gleam/http/response.{Response}
import gleam/io
import gleam/json
import gleam/option.{None}
import gleam/result
import lustre/effect.{type Effect}
import modem
import rsvp

import actions
import core.{type LoginData, type User, LoggedIn, TryingLogin}
import models.{type AppMsg, type Model, Model}
import routes.{HomePage}

pub type LoginValidationDetail {
  LoginFailureDetail(message: String, email: String, password: String)
}

pub fn handle_login_submission(
  model: Model,
  form: Result(LoginData, formlib.Form(LoginData)),
) -> #(Model, Effect(AppMsg)) {
  case form {
    Ok(login_data) -> {
      io.println("Form valid")
      // Form is validated, call API
      #(model, actions.login_via_api(login_data))
    }
    Error(form) -> {
      io.println("Form invalid")
      echo formlib.all_errors(form)
      let model = Model(..model, login_state: TryingLogin(form))
      #(model, effect.none())
    }
  }
}

pub fn handle_login_api_result(
  model: Model,
  res: Result(User, rsvp.Error),
) -> #(Model, Effect(a)) {
  case res {
    Ok(user) -> {
      let login_state = LoggedIn(user)
      // User has logged-in successfully. Redirect to home page
      let #(p, q) = routes.to_uri_parts(HomePage)
      let whatsnext = modem.push(routes.prefix(p, model.mounted_path), q, None)
      let model = Model(..model, login_state:)
      #(model, whatsnext)
    }
    Error(err) -> {
      let detail = case err {
        rsvp.HttpError(Response(body:, status:, ..)) if status == 422 -> {
          let fields_decoder = {
            use email <- decode.field("email", decode.string)
            use password <- decode.field("password", decode.string)
            decode.success(#(email, password))
          }
          let validation_error_decoder = {
            use fields <- decode.field("fields", fields_decoder)
            use message <- decode.field("message", decode.string)
            decode.success(LoginFailureDetail(message, fields.0, fields.1))
          }
          json.parse(body, validation_error_decoder)
          |> result.unwrap(LoginFailureDetail("Some error", "", ""))
        }
        _ -> LoginFailureDetail("Some error", "", "")
      }
      let login_state = case model.login_state {
        TryingLogin(form) -> {
          let form =
            form
            |> formlib.add_error("email", formlib.CustomError(detail.email))
            |> formlib.add_error(
              "password",
              formlib.CustomError(detail.password),
            )
          TryingLogin(form)
        }
        s -> s
      }
      let model = Model(..model, login_state:)
      #(model, effect.none())
    }
  }
}
