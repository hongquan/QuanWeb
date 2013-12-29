from datetime import datetime
from quanweb.common import db
from quanweb.models import ModelMixIn
from auth.models import User


class Author(ModelMixIn, db.Model):
    __tablename__ = 'authors'

    name = db.Column(db.Text, nullable=False)

    def __str__(self):
        return self.name


class Book(ModelMixIn, db.Model):
    __tablename__ = 'books'

    title = db.Column(db.Text, nullable=False)
    download_url = db.Column(db.Text)
    author_id = db.Column(db.Integer, db.ForeignKey('authors.id'))
    author = db.relationship(Author, backref=db.backref('books', lazy='dynamic'))
    user_id = db.Column(db.Integer, db.ForeignKey('users.id'))
    user = db.relationship(User, backref=db.backref('books', lazy='dynamic'))

    date_created = db.Column(db.DateTime, default=datetime.utcnow)
    date_modified = db.Column(db.DateTime, default=datetime.utcnow,
                              onupdate=datetime.utcnow)

    def __str__(self):
        return self.title
