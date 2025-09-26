import gleam/int
import gleam/json
import gleam/option.{Some}
import gleam/uri
import lustre/effect.{type Effect}
import rsvp

import consts
import core.{type LoginData, type Msg, LoginData}
import decoders.{make_user_decoder}

pub fn login_via_api(login_data: LoginData) -> Effect(Msg(r)) {
  let LoginData(email:, password:) = login_data
  let post_data =
    json.object([
      #("email", json.string(email)),
      #("password", json.string(password)),
    ])
  let user_decoder = make_user_decoder()
  let handler = rsvp.expect_json(user_decoder, core.ApiLoginReturned)
  rsvp.post(consts.api_login, post_data, handler)
}

pub fn load_posts(page: Int) -> Effect(Msg(a)) {
  let response_decoder =
    decoders.make_listing_api_decoder(decoders.make_post_decoder())
  let handler = rsvp.expect_json(response_decoder, core.ApiReturnedPosts)
  let query = uri.query_to_string([#("page", int.to_string(page))]) |> Some
  let url = uri.Uri(..uri.empty, path: consts.api_posts, query:)
  rsvp.get(uri.to_string(url), handler)
}

pub fn initiate_logout() -> Effect(Msg(b)) {
  let handler = rsvp.expect_text(core.ApiReturnedLogOutDone)
  rsvp.post("/_api/logout", json.bool(True), handler)
}
