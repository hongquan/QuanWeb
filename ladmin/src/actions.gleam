import gleam/int
import gleam/json
import gleam/option.{type Option, None, Some}
import gleam/uri.{type Uri}
import lustre/effect.{type Effect}
import rsvp

import consts
import core.{
  type LoginData, type Msg, ApiReturnedSinglePost, ApiReturnedSlug, LoginData,
}
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

pub fn load_posts(
  page: Int,
  q: Option(String),
  cat_id: Option(String),
) -> Effect(Msg(a)) {
  let response_decoder =
    decoders.make_listing_api_decoder(decoders.make_post_decoder())
  let handler = rsvp.expect_json(response_decoder, core.ApiReturnedPosts)
  let query_list = [#("page", int.to_string(page))]
  let query_list = case q {
    Some(q) -> [#("q", q), ..query_list]
    None -> query_list
  }
  let query_list = case cat_id {
    Some(q) -> [#("cat_id", q), ..query_list]
    None -> query_list
  }
  let query = uri.query_to_string(query_list) |> Some
  let url = uri.Uri(..uri.empty, path: consts.api_posts, query:)
  rsvp.get(uri.to_string(url), handler)
}

pub fn initiate_logout() -> Effect(Msg(b)) {
  let handler = rsvp.expect_text(core.ApiReturnedLogOutDone)
  rsvp.post("/_api/logout", json.bool(True), handler)
}

pub fn load_categories(page: Int) {
  let response_decoder =
    decoders.make_listing_api_decoder(decoders.make_category_decoder())
  let handler = rsvp.expect_json(response_decoder, core.ApiReturnedCategories)
  let query = uri.query_to_string([#("page", int.to_string(page))]) |> Some
  let url = uri.Uri(..uri.empty, path: consts.api_categories, query:)
  rsvp.get(uri.to_string(url), handler)
}

pub fn load_categories_by_url(url: Uri) -> Effect(Msg(d)) {
  let response_decoder =
    decoders.make_listing_api_decoder(decoders.make_category_decoder())
  let handler = rsvp.expect_json(response_decoder, core.ApiReturnedCategories)
  rsvp.get(uri.to_string(url), handler)
}

pub fn load_single_post(id: String) -> Effect(Msg(c)) {
  let handler =
    rsvp.expect_json(decoders.make_post_decoder(), ApiReturnedSinglePost)
  rsvp.get(consts.api_posts <> id, handler)
}

pub fn initiate_generate_slug(title: String) -> Effect(Msg(e)) {
  let handler = rsvp.expect_text(ApiReturnedSlug)
  rsvp.post(consts.api_slug_generator, json.string(title), handler)
}
