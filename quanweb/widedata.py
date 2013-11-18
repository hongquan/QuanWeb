from flask import g, session

from blog.models import Category
from .common import app
from .models import User

@app.before_request
def authenticate():
    g.user = None
    if 'user_id' in session:
        g.user = User.query.get(session['user_id'])


@app.before_request
def get_categories():
    g.categories = Category.query.all()
