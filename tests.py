from flask import current_app

from attest import Tests, Assert

from quanweb import create_app
from quanweb.models import db, User

TESTING = True
SQLALCHEMY_DATABASE_URI = "sqlite://"
SQLALCHEMY_ECHO = False


def create_user(**kwargs):

    user = User(**kwargs)
    db.session.add(user)
    db.session.commit()
    return user


suite = Tests()

@suite.context
def request_context():
    
    app = create_app(__name__)
    ctx = app.test_request_context()    
    ctx.push()
    try:
        yield 
    finally:
        ctx.pop()


@suite.context
def init_db():

    db.create_all()

    try:
        yield
    finally:
        db.drop_all()
        db.session.remove()


@suite.context
def test_client():
    
    yield current_app.test_client()


@suite.test
def authenticate_invalid_user():
    
    user = User.query.authenticate("ng.hong.quan@gmail.com", "test")
    Assert(user).is_(None)


@suite.test
def authenticate_valid_user():
    
    user = create_user(email="ng.hong.quan@gmail.com", password="test")
    user_id = user.id
    user = User.query.authenticate("ng.hong.quan@gmail.com", "test")
    Assert(user.id) == user_id


@suite.test
def set_user_password():

    user = User(password="test")
    Assert(user.password).is_not("test")


@suite.test
def check_valid_password():

    user = create_user(email="ng.hong.quan@gmail.com", password="test")
    Assert(user.check_password("test")).is_(True)


@suite.test
def check_invalid_password():

    user = create_user(email="ng.hong.quan@gmail.com", password="test")
    Assert(user.check_password("TEST")).is_(False)


@suite.test
def index_not_logged_in(client):
    response = client.get("/")
    Assert(response.status_code) == 200


