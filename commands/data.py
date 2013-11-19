from flask.ext.script import Manager, prompt_bool
from quanweb import app, db
from quanweb.models import User
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
