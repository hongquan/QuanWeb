import logging
from logentries import LogentriesHandler
from cherrypy import wsgiserver

from quanweb import app
from quanweb.config import LOGENTRIES_TOKEN

ADDRESS = '127.0.0.1'
PORT = 2750

handler = LogentriesHandler(LOGENTRIES_TOKEN)
app.logger.addHandler(handler)
logging.getLogger('sqlalchemy').addHandler(handler)

d = wsgiserver.WSGIPathInfoDispatcher({'/': app})
server = wsgiserver.CherryPyWSGIServer((ADDRESS, PORT), d)

if __name__ == '__main__':
    try:
        server.start()
    except KeyboardInterrupt:
        server.stop()
