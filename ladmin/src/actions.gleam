import gleam/dynamic/decode
import gleam/http
import gleam/http/request
import gleam/int
import gleam/json
import gleam/option.{type Option, None, Some}
import gleam/result
import gleam/uri.{type Uri}
import lustre/effect.{type Effect}
import rsvp

import consts
import core.{
  type LoginData, type Msg, type PostEditablePart, ApiCreatedPost,
  ApiRenderedMarkdown, ApiReturnedSinglePost, ApiReturnedSlug, ApiReturnedUsers,
  ApiUpdatedPost, LoginData,
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
    decoders.make_listing_api_decoder(decoders.mini_post_decoder())
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

pub fn update_post_via_api(
  id: String,
  data: PostEditablePart,
  stay: Bool,
) -> Effect(Msg(f)) {
  let body = dump_post_to_json(data) |> json.to_string
  let decoder = decoders.make_post_decoder()
  let handler = rsvp.expect_json(decoder, ApiUpdatedPost(_, stay))
  let url = consts.api_posts <> id
  case
    rsvp.parse_relative_uri(url)
    |> result.try(request.from_uri)
    |> result.map(request.set_header(_, "content-type", "application/json"))
    |> result.map(request.set_method(_, http.Patch))
    |> result.map(request.set_body(_, body))
    |> result.map(rsvp.send(_, handler))
    |> result.map_error(fn(_e) {
      use dispatch <- effect.from
      dispatch(ApiUpdatedPost(Error(rsvp.BadUrl(url)), stay))
    })
  {
    Ok(x) -> x
    Error(x) -> x
  }
}

pub fn create_post_via_api(data: PostEditablePart) {
  let body = dump_post_to_json(data)
  let decoder = decoders.make_post_decoder()
  let handler = rsvp.expect_json(decoder, ApiCreatedPost)
  rsvp.post(consts.api_posts, body, handler)
}

fn dump_post_to_json(post: PostEditablePart) -> json.Json {
  json.object([
    #("title", json.string(post.title)),
    #("slug", json.string(post.slug)),
    #("categories", post.categories |> json.array(json.string)),
    #("body", json.nullable(post.body, json.string)),
    #("locale", json.nullable(post.locale, json.string)),
    #("author", json.string(post.author)),
    #("is_published", json.bool(post.is_published)),
  ])
}

pub fn try_render_markdown_via_api(text: String) -> Effect(Msg(a)) {
  let handler = rsvp.expect_text(ApiRenderedMarkdown)
  let url = consts.api_render_markdown
  case
    rsvp.parse_relative_uri(url)
    |> result.try(request.from_uri)
    |> result.map(request.set_header(_, "content-type", "text/plain"))
    |> result.map(request.set_method(_, http.Post))
    |> result.map(request.set_body(_, text))
    |> result.map(rsvp.send(_, handler))
    |> result.map_error(fn(_e) {
      use dispatch <- effect.from
      dispatch(ApiRenderedMarkdown(Error(rsvp.BadUrl(url))))
    })
  {
    Ok(x) -> x
    Error(x) -> x
  }
}

pub fn load_users() -> Effect(Msg(a)) {
  let response_decoder = decode.list(decoders.mini_user_decoder())
  let handler = rsvp.expect_json(response_decoder, ApiReturnedUsers)
  rsvp.get(consts.api_users, handler)
}
