import iterators
import lustre/attribute as a
import lustre/element/html as h

// Collections: https://loading.io/css/
pub fn render_dense_dots_spinner() {
  let dots =
    iterators.naturals(0)
    |> iterators.take(8)
    |> iterators.map(fn(_n) { h.div([], []) })
  h.div([a.class("lds-roller")], iterators.to_list(dots))
}

// Collections: https://cssloaders.github.io/
pub fn render_three_bar_pulse() {
  h.div([a.class("three-bar-pulse mx-auto")], [])
}
