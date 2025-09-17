import gleam/int
import gleam/json
import gleam/option.{Some}
import gleam/uri
import lustre/effect.{type Effect}
import rsvp

import core.{type LoginData, type Msg, LoginData}
import decoders.{create_user_decoder}

const api_login = "/_api/login"

const api_posts = "/_api/posts"

pub fn login_via_api(login_data: LoginData) -> Effect(Msg(r)) {
  let LoginData(email:, password:) = login_data
  let post_data =
    json.object([
      #("email", json.string(email)),
      #("password", json.string(password)),
    ])
  let user_decoder = create_user_decoder()
  let handler = rsvp.expect_json(user_decoder, core.ApiLoginReturned)
  rsvp.post(api_login, post_data, handler)
}

pub fn load_posts(page: Int) -> Effect(Msg(a)) {
  let response_decoder =
    decoders.create_listing_api_decoder(decoders.create_post_decoder())
  let handler = rsvp.expect_json(response_decoder, core.ApiReturnedPosts)
  let query = uri.query_to_string([#("page", int.to_string(page))]) |> Some
  let url = uri.Uri(..uri.empty, path: api_posts, query:)
  rsvp.get(uri.to_string(url), handler)
}
