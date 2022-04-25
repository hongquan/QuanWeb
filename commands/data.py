import sys

from flask_script import Manager, prompt_bool
from sqlalchemy.exc import IntegrityError
from quanweb import app, db
from blues.auth.models import User
from blues.blog.models import Category


manager = Manager(app)


@manager.command
def resetdb():
    if prompt_bool('Are you sure'):
        db.drop_all()
        db.create_all()
        u = User(username='hongquan', email='ng.hong.quan@gmail.com',
                 password='123456', is_superuser=True)
        db.session.add(u)
        db.session.commit()


@manager.command
def newuser():
    email = input('Email: ')
    email = email.strip()
    username = input('Username: ')
    username = username.strip()
    passw = input('Password: ')
    sup = prompt_bool('Superuser', True)
    u = User(username=username, email=email, password=passw, is_superuser=sup)
    db.session.add(u)
    db.session.commit()


@manager.command
def newcategory():
    print('Existing categories:')
    for cat in Category.query.all():
        print(' ' * 4, cat)
    sel = input('New category name: ')
    name = sel.strip()
    try:
        cat = Category(title=name)
        db.session.add(cat)
        db.session.commit()
        print('Added category', cat)
    except IntegrityError:
        print('This name existed. Bye.', file=sys.stderr)
