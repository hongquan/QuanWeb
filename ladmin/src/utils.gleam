import glanoid
import gleam/int

pub fn gen_nano_id() -> String {
  case glanoid.make_generator(glanoid.default_alphabet) {
    Ok(func) -> func(4)
    _ -> int.random(1000) |> int.to_base16
  }
}
