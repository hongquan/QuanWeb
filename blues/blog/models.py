from datetime import datetime
from slugify import slugify
from sqlalchemy import event

from quanweb.common import db
from quanweb.models import ModelMixIn
from auth.models import User

from .util import make_excerpt

def generate_slug(context):
    if not context:    # Called on empty form
        return
    return slugify(context.current_parameters['title'])


def generate_excerpt(context):
    if not context:
        return
    body = context.current_parameters['body']
    return make_excerpt(body)


entrycats = db.Table('entrycats',
                     db.Column('category_id', db.Integer,
                               db.ForeignKey('categories.id', ondelete='CASCADE')),
                     db.Column('entry_id', db.Integer,
                               db.ForeignKey('entries.id', ondelete='CASCADE')))


class Category(ModelMixIn, db.Model):
    __tablename__ = 'categories'

    title = db.Column(db.String(50), nullable=False)
    slug = db.Column(db.String(50), unique=True, default=generate_slug)

    def __str__(self):
        return self.title



class Entry(ModelMixIn, db.Model):
    __tablename__ = 'entries'

    title = db.Column(db.Unicode(200), nullable=False)
    slug = db.Column(db.String(200), default=generate_slug)
    body = db.Column(db.Text)
    format = db.Column(db.Enum('md', 'rst', name='format_types'), default='md')
    excerpt = db.Column(db.Text, default=generate_excerpt)

    published = db.Column(db.Boolean, default=False)
    date_published = db.Column(db.DateTime, default=datetime.utcnow)

    author_id = db.Column(db.Integer, db.ForeignKey('users.id'))
    author = db.relationship(User, backref=db.backref('posts', lazy='dynamic'))

    categories = db.relationship(Category, secondary=entrycats,
                                 passive_deletes=True,
                                 backref=db.backref('entries', lazy='dynamic'))

    date_created = db.Column(db.DateTime, default=datetime.utcnow)
    date_modified = db.Column(db.DateTime, default=datetime.utcnow,
                              onupdate=datetime.utcnow)

    @classmethod
    # Get only published entries
    def pub(cls):
        return cls.query.filter_by(published=True)

    def __str__(self):
        return self.title


# Event listener
def update_slug(mapper, connection, target):
    target.slug = slugify(target.title)

event.listen(Category, 'before_update', update_slug)
event.listen(Entry, 'before_update', update_slug)

@event.listens_for(Entry, 'before_update')
def update_excerpt(mapper, connection, target):
    target.excerpt = make_excerpt(target.body)
