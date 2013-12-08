import sys
import os.path

from flask.ext.script import Manager, prompt_bool
from sqlalchemy.exc import IntegrityError
from quanweb import app, db
from auth.models import User
from blog.models import Category, Entry

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


@manager.command
def createsamplepost():
    cat = Category(title='Python')
    db.session.add(cat)
    entry = Entry(title='Python is great')
    entry.body = '''
Example code:

```python
print("Hello word"*5)
print("Done")
```

So *neat*!
    '''
    entry.category = cat
    user = User.query.get(1)
    entry.author = user
    db.session.add(entry)
    db.session.commit()


@manager.command
def importfile(filepath):
    base, ext = os.path.splitext(filepath)
    with open(filepath) as fl:
        content = fl.read()
    # Read first line to find title. To specify title,
    # first line should starts with "#" then title
    title = None
    pos = content.find('\n')
    if pos != -1:
        firstline = content[:pos]
        if firstline.startswith('#'):
            title = firstline[1:].strip()
            # Content will be the rest of file
            content = content[pos+1:]
    # If there is no title in file, use file name as title
    if not title:
        title = os.path.basename(base)
    print('Title:', title)
    ask = 'Category? (Enter to not select)\n'
    choices = {}
    names = {}
    for i, name in db.session.query(Category.id, Category.title).all():
        choices[str(i)] = name

    entry = Entry(title=title, body=content)
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
