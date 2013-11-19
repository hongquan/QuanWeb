from flask import url_for
from .views import blogm, UNCATEGORIZED


@blogm.app_template_filter()
def entry_url(entry):
    date_published = entry.date_published
    year, month = date_published.year, date_published.month
    return url_for('blog.show_post', year=year, month=month, pk=entry.id, slug=entry.slug)


@blogm.app_template_filter()
def category_url(cat):
    if cat is None:
        slug = UNCATEGORIZED
    else:
        slug = cat.slug
    return url_for('blog.list_posts', catslug=slug)
