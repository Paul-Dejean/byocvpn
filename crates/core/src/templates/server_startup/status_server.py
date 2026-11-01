import socket
server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
server.bind(('', 51820))
server.listen(10)
while True:
    connection, _ = server.accept()
    try:
        with open('/tmp/byocvpn-status', 'rb') as status_file:
            connection.sendall(status_file.read())
    finally:
        connection.close()
