import consts
import decoders
import gleam/dynamic/decode.{type Decoder}
import gleam/io
import gleam/json.{type Json}
import gleam/result
import gleam/time/calendar
import gleam/time/timestamp.{type Timestamp}
import plinth/javascript/storage

import core.{type User}

pub type Store {
  Store(user: User, last_auth: Timestamp)
}

fn store_decoder() -> Decoder(Store) {
  use user <- decode.field("user", decoders.user_decoder())
  use last_auth <- decode.optional_field(
    "last_auth",
    timestamp.from_calendar(
      calendar.Date(2000, calendar.January, 1),
      calendar.TimeOfDay(0, 0, 0, 0),
      calendar.local_offset(),
    ),
    decoders.timestamp_decoder(),
  )
  decode.success(Store(user:, last_auth:))
}

fn store_to_json(store: Store) -> Json {
  let Store(user:, last_auth:) = store
  json.object([
    #("user", decoders.encode_user(user)),
    #(
      "last_auth",
      json.string(timestamp.to_rfc3339(last_auth, calendar.local_offset())),
    ),
  ])
}

pub fn load_store() -> Result(Store, Nil) {
  let raw =
    storage.local()
    |> result.map_error(fn(_e) {
      io.println_error("Failed to acquire localStorage!")
    })
    |> result.try(storage.get_item(_, consts.key_store))
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

pub fn load_user() -> Result(User, Nil) {
  load_store() |> result.map(fn(s) { s.user })
}

pub fn save_user(user: User) {
  let last_auth = timestamp.system_time()
  let jstore = store_to_json(Store(user:, last_auth:))
  let raw = json.to_string(jstore)
  storage.local()
  |> result.map_error(fn(_e) {
    io.println_error("Failed to acquire localStorage!")
  })
  |> result.try(storage.set_item(_, consts.key_store, raw))
}

pub fn destroy() {
  storage.local()
  |> result.map_error(fn(_e) {
    io.println_error("Failed to acquire localStorage!")
  })
  |> result.map(storage.remove_item(_, consts.key_store))
  |> result.unwrap(Nil)
}
