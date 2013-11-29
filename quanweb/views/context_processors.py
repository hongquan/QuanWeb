from flask import request
from ..common import app
from .. import config

@app.context_processor
def inject_config():
    return {'config': config}

@app.context_processor
def is_running_locally():
    hostname, port = request.host.split(':')
    if port == '5000':
        return {'running_locally': True}
    return {}
