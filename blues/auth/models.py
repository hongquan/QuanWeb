from datetime import date

from werkzeug import generate_password_hash, check_password_hash

from flask.ext.sqlalchemy import BaseQuery
from flask.ext.login import AnonymousUserMixin

from quanweb.common import db


class UserQuery(BaseQuery):

    def authenticate(self, email, password):

        user = self.filter_by(email=email, is_active=True).first()
        if user and user.check_password(password):
            return user


class User(db.Model):

    __tablename__ = 'users'

    query_class = UserQuery

    id = db.Column(db.Integer, primary_key=True)
    username = db.Column(db.String(16), unique=True, nullable=False)

    firstname = db.Column(db.Unicode(40))
    lastname = db.Column(db.Unicode(40))

    email = db.Column(db.String(200), unique=True, nullable=False)

    is_active = db.Column(db.Boolean, default=True)
    is_superuser = db.Column(db.Boolean, default=False)

    _password = db.Column('password', db.String(80), nullable=False)

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
