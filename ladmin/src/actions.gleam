import gleam/dynamic/decode
import gleam/json
import lustre/effect.{type Effect}
import rsvp

import core.{type LoginData, type Msg, LoginData, User}

const api_login = "/_api/login"

pub fn login_via_api(login_data: LoginData) -> Effect(Msg(r)) {
  let LoginData(email:, password:) = login_data
  let post_data =
    json.object([
      #("email", json.string(email)),
      #("password", json.string(password)),
    ])
  let user_decoder = {
    use id <- decode.field("id", decode.string)
    use email <- decode.field("email", decode.string)
    use username <- decode.field("username", decode.string)
    use is_active <- decode.field("is_active", decode.bool)
    use is_superuser <- decode.field("is_superuser", decode.bool)
    decode.success(User(id, email, username, is_active, is_superuser))
  }
  let handler = rsvp.expect_json(user_decoder, core.ApiLoginReturned)
  rsvp.post(api_login, post_data, handler)
}
