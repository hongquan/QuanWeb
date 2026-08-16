import constant
import decoder
import gleam/dynamic/decode.{type Decoder}
import gleam/io
import gleam/json.{type Json}
import gleam/result
import gleam/time/calendar
import gleam/time/timestamp.{type Timestamp}
import plinth/javascript/storage

import core.{type DraftPost, type User}

pub type Store {
  Store(user: User, last_auth: Timestamp, draft_post: DraftPost)
}

fn store_decoder() -> Decoder(Store) {
  use user <- decode.field("user", decoder.user_decoder())
  use last_auth <- decode.optional_field(
    "last_auth",
    timestamp.from_calendar(
      calendar.Date(2000, calendar.January, 1),
      calendar.TimeOfDay(0, 0, 0, 0),
      calendar.local_offset(),
    ),
    decoder.timestamp_decoder(),
  )
  use draft_post <- decode.field("draft_post", core.draft_post_decoder())
  decode.success(Store(user:, last_auth:, draft_post:))
}

fn store_to_json(store: Store) -> Json {
  let Store(user:, last_auth:, draft_post:) = store
  json.object([
    #("user", decoder.encode_user(user)),
    #(
      "last_auth",
      json.string(timestamp.to_rfc3339(last_auth, calendar.local_offset())),
    ),
    #("draft_post", core.draft_post_to_json(draft_post)),
  ])
}

pub fn load_store() -> Result(Store, Nil) {
  let raw =
    storage.local()
    |> result.map_error(fn(_e) {
      io.println_error("Failed to acquire localStorage!")
    })
    |> result.try(storage.get_item(_, constant.key_store))
  case raw {
    Ok(s) ->
      json.parse(s, store_decoder())
      |> result.map_error(fn(e) {
        io.print_error("Failed to deserialize store.")
        echo e
        Nil
      })
    Error(x) -> Error(x)
  }
}

pub fn load_user() -> Result(#(User, Timestamp), Nil) {
  load_store() |> result.map(fn(s) { #(s.user, s.last_auth) })
}

pub fn save_user(user: User) {
  let last_auth = timestamp.system_time()
  let saved_draft =
    load_store()
    |> result.map(fn(s) { s.draft_post })
    |> result.unwrap(core.DraftPost(id: "", body: ""))
  let jstore = store_to_json(Store(user:, last_auth:, draft_post: saved_draft))
  let raw = json.to_string(jstore)
  storage.local()
  |> result.map_error(fn(_e) {
    io.println_error("Failed to acquire localStorage!")
  })
  |> result.try(storage.set_item(_, constant.key_store, raw))
}

pub fn destroy() {
  storage.local()
  |> result.map_error(fn(_e) {
    io.println_error("Failed to acquire localStorage!")
  })
  |> result.map(storage.remove_item(_, constant.key_store))
  |> result.unwrap(Nil)
}

pub fn save_draft_post(draft: DraftPost) -> Result(Nil, Nil) {
  use existing <- result.try(load_store())
  let Store(user:, last_auth:, ..) = existing
  let jstore = store_to_json(Store(user:, last_auth:, draft_post: draft))
  let raw = json.to_string(jstore)
  storage.local()
  |> result.map_error(fn(_e) {
    io.println_error("Failed to acquire localStorage!")
  })
  |> result.try(storage.set_item(_, constant.key_store, raw))
}

pub fn load_draft_post() -> Result(DraftPost, Nil) {
  use store <- result.try(load_store())
  let Store(draft_post:, ..) = store
  Ok(draft_post)
}

// Remove everything, except `draft_post` from the store.
pub fn delete_authentication() {
  use existing <- result.try(load_store())
  let jstore =
    json.object([#("draft_post", core.draft_post_to_json(existing.draft_post))])
  let raw = json.to_string(jstore)
  storage.local()
  |> result.map_error(fn(_e) {
    io.println_error("Failed to acquire localStorage!")
  })
  |> result.try(storage.set_item(_, constant.key_store, raw))
}
