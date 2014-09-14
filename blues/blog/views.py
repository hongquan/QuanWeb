from jinja2 import TemplateNotFound
from sqlalchemy.orm import load_only
from flask import Blueprint, render_template, abort

from quanweb import config
from quanweb.common import UNCATEGORIZED
from .models import db, Entry, Category

blogm = Blueprint('blog', __name__, static_folder=config.STATIC_FOLDER,
                  template_folder=config.TEMPLATE_FOLDER)


@blogm.route('/<int:year>/<int:month>/<int:pk>/<slug>')
def show_post(year, month, pk, slug):
    entry = Entry.query.get(pk)
    siblings = Entry.query.options(load_only('id', 'date_published'))
    next_entry = siblings.filter(Entry.id>pk).first()
    prev_entry = siblings.filter(Entry.id<pk).order_by('-id').first()
    return render_template('blog/entry.html', entry=entry,
                           prev_entry=prev_entry, next_entry=next_entry)


@blogm.route('/<catslug>')
@blogm.route('/')
def list_posts(catslug=None):
    cvars = {}
    query = Entry.pub().order_by(Entry.date_published.desc())
    if catslug == UNCATEGORIZED:
        entries = query.filter_by(categories=None)
    elif catslug:
        category = Category.query.filter_by(slug=catslug).one()
        cvars['cat'] = category
        entries = category.entries.all()
    else:
        entries = query.all()
    cvars['entries'] = entries
    return render_template('blog/entries.html', **cvars)
