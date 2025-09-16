import gleam/dynamic/decode.{type Decoder}
import gleam/json

import core.{type User, User}

pub fn get_user_decoder() -> Decoder(User) {
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
