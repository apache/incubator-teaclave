import socket
import struct
import ssl
import threading

hostname = 'localhost'
context = ssl._create_unverified_context()

def login():
    with socket.create_connection((hostname, 7777)) as sock:
        with context.wrap_socket(sock, server_hostname=hostname) as ssock:
            message = b'{"type":"user_login","id":"20937006-2718-4f33-bae2-567933807436","password":"d20ce53ab743d69320712fd98555f5e5"}'
            ssock.write(struct.pack(">Q", len(message)))
            ssock.write(message)

            response_len = struct.unpack(">Q", ssock.read(8))
            response = ssock.read(response_len[0])
            print(response)

for i in range(100):
    threading.Thread(target=login).start()
