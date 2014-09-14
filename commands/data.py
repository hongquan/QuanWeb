import sys
import os.path

from flask_script import Manager, prompt_bool
from sqlalchemy.exc import IntegrityError
from quanweb import app, db
from auth.models import User
from blog.models import Category, Entry

from .tools import split_content

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
        print(' '*4, cat)
    sel = input('New category name: ')
    name = sel.strip()
    try:
        cat = Category(title=name)
        db.session.add(cat)
        db.session.commit()
        print('Added category', cat)
    except IntegrityError:
        print('This name existed. Bye.', file=sys.stderr)


@manager.option(dest='filepath', help='Input file path')
@manager.option('-u', dest='pid', help='Post ID to update. Will create new post if not specified')
def importfile(filepath, pid=None):
    base, ext = os.path.splitext(filepath)
    title, content = split_content(filepath)
    print('Title:', title)
    # Update existing post, if pid is specified
    if pid:
        existing = Entry.query.get(pid)
        if not existing:
            print('Post with ID {} is not exist'.format(pid), file=sys.stderr)
            sys.exit(-1)
        # Found
        print('To update post {}: {}'.format(pid, existing.title))
        existing.title = title
        existing.body = content
        db.session.commit()
        print('Updated post', existing.title)
        return
    # Create new post
    ask = 'Category? (Enter to not select)\n'
    choices = {}
    names = {}
    for i, name in db.session.query(Category.id, Category.title).all():
        choices[str(i)] = name

    entry = Entry(title=title, body=content, published=True)
    while choices.keys():
        question = ask + '\n'.join('{}. {} '.format(k, v) for k, v in choices.items())
        sel = input(question)
        if sel != '' and sel not in choices:
            print('Wrong choice. Bye.', file=sys.stderr)
            continue
        elif not sel:
            break;
        cat = Category.query.get(sel)
        entry.categories.append(cat)
        del choices[sel]
    db.session.add(entry)
    db.session.commit()
    print('Added your post', title)
