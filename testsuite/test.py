import socket
import struct
import ssl
import threading

hostname = 'localhost'
context = ssl._create_unverified_context()

def login(user_id, user_password):
    with socket.create_connection((hostname, 7777)) as sock:
        with context.wrap_socket(sock, server_hostname=hostname) as ssock:
            message = '{"type":"user_login","id":"'+ user_id + '","password":"' + user_password + '"}'
            message = message.encode()
            ssock.write(struct.pack(">Q", len(message)))
            ssock.write(message)

            response_len = struct.unpack(">Q", ssock.read(8))
            response = ssock.read(response_len[0])
            print(response)

for i in range(100):
    threading.Thread(target=login, args=("test_id", "test_password")).start()

login("invalid_id", "invalid_password")
