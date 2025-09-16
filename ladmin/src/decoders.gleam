import gleam/dynamic/decode.{type Decoder}

import core.{type User, User}

pub fn get_user_decoder() -> Decoder(User) {
  use id <- decode.field("id", decode.string)
  use email <- decode.field("email", decode.string)
  use username <- decode.field("username", decode.string)
  use is_active <- decode.field("is_active", decode.bool)
  use is_superuser <- decode.field("is_superuser", decode.bool)
  decode.success(User(id, email, username, is_active, is_superuser))
}
