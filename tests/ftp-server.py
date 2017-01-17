import os

from pyftpdlib.authorizers import DummyAuthorizer
from pyftpdlib.handlers import TLS_FTPHandler
from pyftpdlib.servers import FTPServer
from pyftpdlib.filesystems import AbstractedFS

authorizer = DummyAuthorizer()
authorizer.add_anonymous(os.getcwd())

handler = TLS_FTPHandler
handler.keyfile = './test.key'
handler.certfile = './test.crt'
handler.authorizer = authorizer
handler.passive_ports = range(60000, 65535)

# Instantiate FTP server class and listen on 0.0.0.0:21
address = ('', 21)
server = FTPServer(address, handler)

# start ftp server
server.serve_forever()
