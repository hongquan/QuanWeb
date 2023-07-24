
from .views import bookshelfm, BookListView
from . import models

bookshelfm.add_url_rule('/', view_func=BookListView.as_view('list'))
