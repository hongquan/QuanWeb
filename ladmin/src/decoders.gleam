import gleam/dynamic.{type Dynamic}
import gleam/dynamic/decode.{type Decoder}
import gleam/json
import gleam/result
import gleam/time/timestamp
import gleam/uri
import tempo.{type DateTime}
import tempo/datetime

import core.{
  type Category, type MiniPost, type User, ApiListingResponse, Category,
  MiniPost, Post, User,
}

pub fn make_user_decoder() -> Decoder(User) {
  use id <- decode.field("id", decode.string)
  use email <- decode.field("email", decode.string)
  use username <- decode.field("username", decode.string)
  use is_active <- decode.field("is_active", decode.bool)
  use is_superuser <- decode.field("is_superuser", decode.bool)
  decode.success(User(id, email, username, is_active, is_superuser))
}

pub fn encode_user(user: User) -> json.Json {
  json.object([
    #("id", json.string(user.id)),
    #("email", json.string(user.email)),
    #("username", json.string(user.username)),
    #("is_active", json.bool(user.is_active)),
    #("is_superuser", json.bool(user.is_superuser)),
  ])
}

pub fn decode_datetime(
  data: Dynamic,
) -> Result(DateTime, List(decode.DecodeError)) {
  data
  |> decode.run(decode.string)
  |> result.try(fn(s) {
    timestamp.parse_rfc3339(s)
    |> result.map_error(fn(_e) { decode.decode_error("RFC-3339 string", data) })
    |> result.map(datetime.from_timestamp)
  })
}

pub fn make_datetime_decoder() -> Decoder(DateTime) {
  use data <- decode.new_primitive_decoder("DateTime")
  data |> decode_datetime |> result.replace_error(datetime.unix_epoch)
}

pub fn make_uri_decoder() -> Decoder(uri.Uri) {
  use data <- decode.new_primitive_decoder("Uri")
  data
  |> decode.run(decode.string)
  |> result.try(fn(s) {
    uri.parse(s)
    |> result.map_error(fn(_e) { decode.decode_error("URI string", data) })
  })
  |> result.replace_error(uri.empty)
}

pub fn mini_post_decoder() -> decode.Decoder(MiniPost) {
  use id <- decode.field("id", decode.string)
  use title <- decode.field("title", decode.string)
  use slug <- decode.field("slug", decode.string)
  use is_published <- decode.field("is_published", decode.bool)
  let datetime_decoder = make_datetime_decoder()
  use created_at <- decode.field("created_at", datetime_decoder)
  use updated_at <- decode.field("updated_at", datetime_decoder)
  let category_decoder = make_category_decoder()
  use categories <- decode.field("categories", decode.list(category_decoder))
  decode.success(MiniPost(
    id:,
    title:,
    slug:,
    is_published:,
    created_at:,
    updated_at:,
    categories:,
  ))
}

pub fn make_post_decoder() -> Decoder(core.Post) {
  use id <- decode.field("id", decode.string)
  use title <- decode.field("title", decode.string)
  use slug <- decode.field("slug", decode.string)
  use body <- decode.field("body", decode.string)
  use is_published <- decode.field("is_published", decode.bool)
  let datetime_decoder = make_datetime_decoder()
  use created_at <- decode.field("created_at", datetime_decoder)
  use updated_at <- decode.field("updated_at", datetime_decoder)
  let category_decoder = make_category_decoder()
  use categories <- decode.field("categories", decode.list(category_decoder))
  decode.success(Post(
    id:,
    title:,
    slug:,
    body:,
    is_published:,
    created_at:,
    updated_at:,
    categories:,
  ))
}

pub fn make_category_decoder() -> Decoder(Category) {
  use id <- decode.field("id", decode.string)
  use title <- decode.field("title", decode.string)
  use slug <- decode.field("slug", decode.string)
  use title_vi <- decode.field("title_vi", decode.optional(decode.string))
  decode.success(Category(id:, title:, slug:, title_vi:))
}

pub fn make_listing_api_decoder(
  object_decoder: Decoder(o),
) -> Decoder(core.ApiListingResponse(o)) {
  use count <- decode.field("count", decode.int)
  use total_pages <- decode.field("total_pages", decode.int)
  let uri_decoder = make_uri_decoder()
  use prev <- decode.subfield(["links", "prev"], decode.optional(uri_decoder))
  use next <- decode.subfield(["links", "next"], decode.optional(uri_decoder))
  use objects <- decode.field("objects", decode.list(object_decoder))
  decode.success(
    ApiListingResponse(count:, objects:, total_pages:, links: #(prev, next)),
  )
}
