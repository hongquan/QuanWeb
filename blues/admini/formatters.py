from quanweb.utils import truncate_text

def truncate_longtext(view, context, model, name):
    return truncate_text(getattr(model, name))

def email_nohost(view, context, model, name):
    full = getattr(model, name)
    if not full:
        return ''
    return str(full).split('@')[0]