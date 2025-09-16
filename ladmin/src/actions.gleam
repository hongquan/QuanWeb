import gleam/json
import lustre/effect.{type Effect}
import rsvp

import core.{type LoginData, type Msg, LoginData}
import decoders.{get_user_decoder}

const api_login = "/_api/login"

pub fn login_via_api(login_data: LoginData) -> Effect(Msg(r)) {
  let LoginData(email:, password:) = login_data
  let post_data =
    json.object([
      #("email", json.string(email)),
      #("password", json.string(password)),
    ])
  let user_decoder = get_user_decoder()
  let handler = rsvp.expect_json(user_decoder, core.ApiLoginReturned)
  rsvp.post(api_login, post_data, handler)
}
