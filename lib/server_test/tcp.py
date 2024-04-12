import socket
import threading

TCP_IP = "127.0.0.1"
TCP_PORT = 5007

def handle_client(conn, addr):
    print(f"Connected by {addr}")
    data = conn.recv(1024)
    print(f"Received message: {data.decode()}")
    response = f"Server received: {data.decode()}"
    conn.send(response.encode())
    conn.close()

sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
sock.bind((TCP_IP, TCP_PORT))
sock.listen(5)

while True:
    conn, addr = sock.accept()
    client_thread = threading.Thread(target=handle_client, args=(conn, addr))
    client_thread.start()