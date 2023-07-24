from flask import Blueprint
from flask import render_template
from flask.views import MethodView

from quanweb import config
from .models import Book

bookshelfm = Blueprint('bookshelf', __name__,
                       static_folder=config.STATIC_FOLDER,
                       template_folder=config.TEMPLATE_FOLDER)


class BookListView(MethodView):
    def get(self):
        books = Book.query.all()
        return render_template('bookshelf/booklist.jinja', books=books)
