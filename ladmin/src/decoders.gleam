import gleam/dynamic.{type Dynamic}
import gleam/dynamic/decode.{type Decoder}
import gleam/json
import gleam/result
import gleam/time/timestamp
import gleam/uri
import tempo.{type DateTime}
import tempo/datetime

import core.{type User, ApiListingResponse, Post, User}

pub fn create_user_decoder() -> Decoder(User) {
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

pub fn create_datetime_decoder() {
  use data <- decode.new_primitive_decoder("DateTime")
  data |> decode_datetime |> result.replace_error(datetime.unix_epoch)
}

pub fn create_uri_decoder() {
  use data <- decode.new_primitive_decoder("Uri")
  data
  |> decode.run(decode.string)
  |> result.try(fn(s) {
    uri.parse(s)
    |> result.map_error(fn(_e) { decode.decode_error("URI string", data) })
  })
  |> result.replace_error(uri.empty)
}

pub fn create_post_decoder() {
  use id <- decode.field("id", decode.string)
  use title <- decode.field("title", decode.string)
  use slug <- decode.field("slug", decode.string)
  use is_published <- decode.field("is_published", decode.bool)
  let datetime_decoder = create_datetime_decoder()
  use created_at <- decode.field("created_at", datetime_decoder)
  decode.success(Post(id:, title:, slug:, is_published:, created_at:))
}

pub fn create_listing_api_decoder(
  object_decoder: Decoder(o),
) -> Decoder(core.ApiListingResponse(o)) {
  use count <- decode.field("count", decode.int)
  use total_pages <- decode.field("total_pages", decode.int)
  let uri_decoder = create_uri_decoder()
  use prev <- decode.subfield(["links", "prev"], decode.optional(uri_decoder))
  use next <- decode.subfield(["links", "next"], decode.optional(uri_decoder))
  use objects <- decode.field("objects", decode.list(object_decoder))
  decode.success(
    ApiListingResponse(count:, objects:, total_pages:, links: #(prev, next)),
  )
}
