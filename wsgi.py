import logging
from logentries import LogentriesHandler

from quanweb import app
from quanweb.config import APPENLIGHT_PRVKEY, LOGENTRIES_TOKEN

ADDRESS = '127.0.0.1'
PORT = 2750

if LOGENTRIES_TOKEN:
    handler = LogentriesHandler(LOGENTRIES_TOKEN)
    app.logger.addHandler(handler)
    logging.getLogger('sqlalchemy').addHandler(handler)

