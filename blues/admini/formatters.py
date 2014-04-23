import arrow
from htmllaundry import strip_markup
from quanweb.utils import truncate_text

def truncate_longtext(view, context, model, name):
    return truncate_text(getattr(model, name))

def truncate_html(view, context, model, name):
    return truncate_text(strip_markup(getattr(model, name)), 80)

def email_nohost(view, context, model, name):
    full = getattr(model, name)
    if not full:
        return ''
    return str(full).split('@')[0]

def datetime_short(view, value):
    return arrow.get(value).format('D MMM YYYY h:ma')
