import os.path
import logging
import cherrypy
import wsgilog

from os.path import dirname, abspath
from logging.handlers import RotatingFileHandler
from cherrypy import wsgiserver

from quanweb import app

ADDRESS = '127.0.0.1'
PORT = 2750

LOG_ERROR = os.path.join(dirname(dirname(abspath(__file__))), 'quanweb.error.log')

cherrypy.config.update({'log.wsgi': True})
logged_app = wsgilog.WsgiLog(app, tohtml=True, tofile=True, file=LOG_ERROR)
file_handler = RotatingFileHandler(LOG_ERROR, maxBytes=1024*1024*100,
                                   backupCount=2)
app.logger.addHandler(file_handler)
logging.getLogger('sqlalchemy').addHandler(file_handler)
d = wsgiserver.WSGIPathInfoDispatcher({'/': logged_app})
server = wsgiserver.CherryPyWSGIServer((ADDRESS, PORT), d)

if __name__ == '__main__':
    try:
        server.start()
    except KeyboardInterrupt:
        server.stop()
