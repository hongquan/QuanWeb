from datetime import datetime
from slugify import slugify

from quanweb.common import db, md
from auth.models import User


def generate_slug(context):
    return slugify(context.current_parameters['title'])


def generate_preamble(context):
    body = context.current_parameters['body']
    lines = body.splitlines(True)[:7]
    # Count "code block" marker (```)
    count = sum(1 for l in lines if l.startswith('```'))
    if (count % 2) == 1:  # There are odd number of marks
        if lines[-1].startswith('```'):
            # Remove last mark...
            lines = lines[:-1]
        else:
            # ...Or add another mark to make sure the number is even
            lines.append('```')
    reduced = ''.join(lines)
    return md._instance.convert(reduced)


entrycats = db.Table('entrycats',
                     db.Column('category_id', db.Integer, db.ForeignKey('categories.id')),
                     db.Column('entry_id', db.Integer, db.ForeignKey('entries.id')))


class Category(db.Model):
    __tablename__ = 'categories'
    id = db.Column(db.Integer, primary_key=True)
    title = db.Column(db.String(50), nullable=False)
    slug = db.Column(db.String(50), unique=True, default=generate_slug, onupdate=generate_slug)


    def __str__(self):
        return self.title



class Entry(db.Model):
    __tablename__ = 'entries'
    id = db.Column(db.Integer, primary_key=True)

    title = db.Column(db.Unicode(200), nullable=False)
    slug = db.Column(db.String(200), default=generate_slug, onupdate=generate_slug)
    body = db.Column(db.Text)
    format = db.Column(db.Enum('md', 'rst', name='format_types'), default='md')
    preamble = db.Column(db.Text, default=generate_preamble, onupdate=generate_preamble)

    published = db.Column(db.Boolean, default=False)
    date_published = db.Column(db.DateTime, default=datetime.utcnow)

    author_id = db.Column(db.Integer, db.ForeignKey('users.id'))
    author = db.relationship(User, backref=db.backref('posts', lazy='dynamic'))

    categories = db.relationship(Category, secondary=entrycats,
                                 backref=db.backref('entries', lazy='dynamic'))

    date_created = db.Column(db.DateTime, default=datetime.utcnow)
    date_modified = db.Column(db.DateTime, default=datetime.utcnow,
                              onupdate=datetime.utcnow)

    def __str__(self):
        return self.title
