import os.path
import gzip

from blog.models import Entry
from .common import loginmanager
from .models import User

@loginmanager.user_loader
def load_user(userid):
    return User.get(userid)


def import_file(filepath):
    base, ext = os.path.splitext(filepath)
    if ext == '.gz':
        with gzip.open(filepath, 'rb') as fl:
            content = fl.read()
    elif ext in ('.md', ''):
        with open(filepath) as fl:
            content = fl.read()
