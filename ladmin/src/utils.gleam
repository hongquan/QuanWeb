import gleam/list
import gleam/string

const default_alphabet = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_-"

const default_size = 8

pub fn gen_simple_random_id() {
  let pool = string.to_graphemes(default_alphabet)
  list.sample(pool, default_size) |> list.shuffle |> string.concat
}
