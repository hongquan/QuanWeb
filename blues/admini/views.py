from flask import Markup
from datetime import datetime
from htmltag import a, span
from urllib.parse import urlencode
from jinja2 import contextfunction
from flask_login import current_user
from flask_admin.model import typefmt
from flask_admin.actions import action
from flask import request, redirect, url_for
from flask_admin.contrib.sqla import ModelView
from flask_admin.base import AdminIndexView, expose

from auth.models import User
from quanweb.common import db
from blog.models import Category, Entry

from .formatters import truncate_longtext, truncate_html, \
                        email_nohost, datetime_short

MY_DEFAULT_FORMATTERS = dict(typefmt.BASE_FORMATTERS)
MY_DEFAULT_FORMATTERS.update({
    type(datetime(2000, 1, 1)): datetime_short
})

class QAdmin(ModelView):
    edit_template = 'admin/edit.html'
    create_template = 'admin/create.html'

    def __init__(self, model, **kwargs):
        super().__init__(model, db.session, **kwargs)

    def is_accessible(self):
        return current_user.is_authenticated()


class AdminHomeView(AdminIndexView):
    @expose('/')
    def index(self):
        if not current_user.is_authenticated():
            url = url_for('login') + '?' + urlencode({'next': request.path})
            return redirect(url)
        return super().index()


class CategoryAdmin(QAdmin):
    form_excluded_columns = ('entries',)

    def __init__(self):
        super().__init__(Category, name='Categories', endpoint='categories')


class EntryAdmin(QAdmin):
    create_template = 'admin/entry_edit.html'
    edit_template = 'admin/entry_edit.html'

    column_formatters = {
        'author': email_nohost,
        'body': truncate_longtext,
        'excerpt': truncate_html
    }
    column_type_formatters = MY_DEFAULT_FORMATTERS
    column_default_sort = (Entry.id, True)
    column_list = ('title', 'excerpt', 'author', 'body',
                   'published', 'date_modified')

    form_excluded_columns = ('slug', 'excerpt', 'date_published',
                             'date_created', 'date_modified')

    def __init__(self):
        super().__init__(Entry, name='Entries', endpoint='entries')

    @action('publish', 'Publish')
    def action_publish(self, ids):
        queryset = Entry.query.filter(Entry.id.in_(ids))
        queryset.update({'published': True}, synchronize_session=False)
        db.session.commit()

    def get_list_columns(self):
        column_list = super().get_list_columns()
        column_list.append(('extra', ''))
        return column_list

    @contextfunction
    def get_list_value(self, context, model, name):
        if name == 'extra':
            date_published = model.date_published
            year, month = date_published.year, date_published.month
            url = url_for('blog.show_post', year=year, month=month,
                          pk=model.id, slug=model.slug)
            link = a(span(_class='glyphicon glyphicon-eye-open'), href=url)
            return Markup(link)
        return super().get_list_value(context, model, name)


class UserAdmin(QAdmin):
    column_exclude_list = ('password', '_password')
    form_excluded_columns = ('password', 'posts', 'books')

    def __init__(self):
        super().__init__(User, name='Users', endpoint='users')
