from urllib.parse import urlencode
from flask import url_for

from ..common import app


app.jinja_env.globals['static'] = lambda filename: url_for('static', filename=filename)

@app.template_filter()
def strftime(date, fmt):
    return date.strftime(fmt)

@app.template_filter()
def add_urlparam(url, key, value):
    try:
        base, params = url.split('?')
    except ValueError:
        base = url
        params = {}
    params[key] = value
    return '{}?{}'.format(base, urlencode(params))

@app.template_filter()
def compare_flip(compared, comparing, value_true, value_false=''):
    if compared == comparing:
        return value_true
    return value_false

@app.template_filter()
def yesno(value_test, value_true, value_false=''):
    if value_test:
        return value_true
    return value_false

