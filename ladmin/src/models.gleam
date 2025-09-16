import core.{type LoginState, type Msg}
import routes.{type Route}

pub type Model {
  Model(mounted_path: String, route: Route, login_state: LoginState)
}

// `Msg` is generic with route type, we make concrete type here
pub type AppMsg =
  Msg(Route)
