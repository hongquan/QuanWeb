
from werkzeug import generate_password_hash, check_password_hash

from flask_sqlalchemy import BaseQuery
from flask_login import AnonymousUserMixin, UserMixin

from quanweb.common import db
from quanweb.models import ModelMixIn


class UserQuery(BaseQuery):

    def authenticate(self, email, password):

        user = self.filter_by(email=email, active=True).first()
        if user and user.check_password(password):
            return user


class User(ModelMixIn, UserMixin, db.Model):

    __tablename__ = 'users'

    query_class = UserQuery

    username = db.Column(db.String(16), unique=True, nullable=False)

    firstname = db.Column(db.Unicode(40))
    lastname = db.Column(db.Unicode(40))

    email = db.Column(db.String(200), unique=True, nullable=False)

    active = db.Column(db.Boolean, default=True)
    is_superuser = db.Column(db.Boolean, default=False)

    _password = db.Column('password', db.String(100), nullable=False)

    def _get_password(self):
        return self._password

    def _set_password(self, password):
        self._password = generate_password_hash(password)

    password = db.synonym('_password',
                          descriptor=property(_get_password, _set_password))

    def check_password(self, password):
        return check_password_hash(self.password, password)

    def __str__(self):
        return self.email


class AnonymousUser(AnonymousUserMixin):
    pass
