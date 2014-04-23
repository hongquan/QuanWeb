from flask_admin.contrib.sqla import ModelView
from flask_login import current_user

from quanweb.common import db
from blog.models import Category, Entry

from .formatters import truncate_longtext, email_nohost

class QAdmin(ModelView):
    edit_template = 'admin/edit.html'
    create_template = 'admin/create.html'

    def __init__(self, model, **kwargs):
        super().__init__(model, db.session, **kwargs)

    def is_accessible(self):
        return current_user.is_authenticated()


class CategoryAdmin(QAdmin):
    form_excluded_columns = ('entries',)

    def __init__(self):
        super().__init__(Category, name='Categories', endpoint='categories')


class EntryAdmin(QAdmin):
    column_formatters = {
        'author': email_nohost,
        'body': truncate_longtext,
        'excerpt': truncate_longtext
    }

    form_excluded_columns = ('slug', 'excerpt', 'date_published', 'date_created', 'date_modified')

    def __init__(self):
        super().__init__(Entry, name='Entries', endpoint='entries')