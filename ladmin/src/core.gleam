import formal/form.{type Form}
import rsvp
import tempo.{type DateTime}

pub type Post {
  Post(
    id: String,
    title: String,
    slug: String,
    is_published: Bool,
    created_at: DateTime,
  )
}

pub type User {
  User(
    id: String,
    email: String,
    username: String,
    is_active: Bool,
    is_superuser: Bool,
  )
}

pub type LoginData {
  LoginData(email: String, password: String)
}

pub type LoginState {
  NonLogin
  TryingLogin(Form(LoginData))
  LoggedIn(User)
}

pub type Msg(r) {
  RouterInitDone
  UserSubmittedLoginForm(Result(LoginData, Form(LoginData)))
  ApiLoginReturned(Result(User, rsvp.Error))
  ApiReturnedPosts
  OnRouteChange(r)
}
