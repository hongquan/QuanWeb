from flask import Blueprint, render_template, abort
from jinja2 import TemplateNotFound

from quanweb import config
from .models import db, Entry, Category

UNCATEGORIZED = '_uncategorized'

blogm = Blueprint('blog', __name__, static_folder=config.STATIC_FOLDER,
                  template_folder=config.TEMPLATE_FOLDER)


@blogm.route('/<int:year>/<int:month>/<int:pk>/<slug>')
def show_post(year, month, pk, slug):
    entry = Entry.query.get(pk)
    return render_template('blog/entry.html', entry=entry)


@blogm.route('/<catslug>')
@blogm.route('/')
def list_posts(catslug=None):
    cvars = {}
    query = Entry.query.order_by(Entry.date_published.desc())
    if catslug == UNCATEGORIZED:
        entries = query.filter_by(category=None)
    elif catslug:
        category = query.filter_by(slug=catslug).one()
        cvars['cat'] = category
        entries = query.filter_by(category=category)
    else:
        entries = query.all()
    cvars['entries'] = entries
    return render_template('blog/entries.html', **cvars)
