import logging
from logentries import LogentriesHandler
import appenlight_client.ext.flask as appenlight

from quanweb import app
from quanweb.config import APPENLIGHT_PRVKEY, LOGENTRIES_TOKEN

ADDRESS = '127.0.0.1'
PORT = 2750

if LOGENTRIES_TOKEN:
    handler = LogentriesHandler(LOGENTRIES_TOKEN)
    app.logger.addHandler(handler)
    logging.getLogger('sqlalchemy').addHandler(handler)

if APPENLIGHT_PRVKEY:
    app = appenlight.add_appenlight(app, {'appenlight.api_key': APPENLIGHT_PRVKEY})

if __name__ == '__main__':
    from waitress import serve
    serve(app, host=ADDRESS, port=PORT)
