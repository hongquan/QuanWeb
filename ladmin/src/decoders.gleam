import gleam/dynamic/decode.{type Decoder}
import gleam/json
import gleam/result
import gleam/time/timestamp.{type Timestamp}
import gleam/uri

import core.{
  type Category, type MiniPost, type MiniUser, type Post, type User,
  ApiListingResponse, Category, MiniPost, MiniUser, Post, User,
}

pub fn user_decoder() -> Decoder(User) {
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

pub fn uri_decoder() -> Decoder(uri.Uri) {
  use data <- decode.new_primitive_decoder("Uri")
  data
  |> decode.run(decode.string)
  |> result.try(fn(s) {
    uri.parse(s)
    |> result.map_error(fn(_e) { decode.decode_error("URI string", data) })
  })
  |> result.replace_error(uri.empty)
}

pub fn timestamp_decoder() -> Decoder(Timestamp) {
  use data <- decode.new_primitive_decoder("Timestamp")
  decode.run(data, decode.string)
  |> result.replace_error(Nil)
  |> result.try(timestamp.parse_rfc3339)
  |> result.replace_error(timestamp.from_unix_seconds(0))
}

pub fn mini_post_decoder() -> Decoder(MiniPost) {
  use id <- decode.field("id", decode.string)
  use title <- decode.field("title", decode.string)
  use slug <- decode.field("slug", decode.string)
  use is_published <- decode.field("is_published", decode.bool)
  use created_at <- decode.field("created_at", timestamp_decoder())
  use updated_at <- decode.field("updated_at", timestamp_decoder())
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

pub fn make_post_decoder() -> Decoder(Post) {
  use id <- decode.field("id", decode.string)
  use title <- decode.field("title", decode.string)
  use slug <- decode.field("slug", decode.string)
  use body <- decode.field("body", decode.optional(decode.string))
  use is_published <- decode.field("is_published", decode.bool)
  use created_at <- decode.field("created_at", timestamp_decoder())
  use updated_at <- decode.field("updated_at", timestamp_decoder())
  let category_decoder = make_category_decoder()
  use categories <- decode.field("categories", decode.list(category_decoder))
  use locale <- decode.field("locale", decode.optional(decode.string))
  use author <- decode.field("author", decode.optional(mini_user_decoder()))
  use og_image <- decode.field("og_image", decode.optional(decode.string))
  use seo_description <- decode.field(
    "seo_description",
    decode.optional(decode.string),
  )
  decode.success(Post(
    id:,
    title:,
    slug:,
    body:,
    is_published:,
    created_at:,
    updated_at:,
    categories:,
    locale:,
    author:,
    og_image:,
    seo_description:,
  ))
}

pub fn make_category_decoder() -> Decoder(Category) {
  use id <- decode.field("id", decode.string)
  use title <- decode.field("title", decode.string)
  use slug <- decode.field("slug", decode.string)
  use title_vi <- decode.field("title_vi", decode.optional(decode.string))
  use header_color <- decode.field(
    "header_color",
    decode.optional(decode.string),
  )
  use featured_order <- decode.field("featured_order", decode.optional(decode.int))
  use summary_en <- decode.field("summary_en", decode.optional(decode.string))
  use summary_vi <- decode.field("summary_vi", decode.optional(decode.string))
  decode.success(Category(id:, title:, slug:, title_vi:, header_color:, featured_order:, summary_en:, summary_vi:))
}

pub fn make_listing_api_decoder(
  object_decoder: Decoder(o),
) -> Decoder(core.ApiListingResponse(o)) {
  use count <- decode.field("count", decode.int)
  use total_pages <- decode.field("total_pages", decode.int)
  let uri_decoder = uri_decoder()
  use prev <- decode.subfield(["links", "prev"], decode.optional(uri_decoder))
  use next <- decode.subfield(["links", "next"], decode.optional(uri_decoder))
  use objects <- decode.field("objects", decode.list(object_decoder))
  decode.success(
    ApiListingResponse(count:, objects:, total_pages:, links: #(prev, next)),
  )
}

pub fn mini_user_decoder() -> Decoder(MiniUser) {
  use id <- decode.field("id", decode.string)
  use email <- decode.field("email", decode.string)
  decode.success(MiniUser(id:, email:))
}
