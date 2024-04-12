from flask import Flask
from flask_socketio import SocketIO, send

app = Flask(__name__)
socketio = SocketIO(app)

@socketio.on('message')
def handle_message(message):
    print(f"Received message: {message}")
    response = f"Server received: {message}"
    send(response, broadcast=True)

if __name__ == '__main__':
    socketio.run(app, host='localhost', port=5001)