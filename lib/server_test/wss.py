import asyncio
import websockets

async def handle_connection(websocket, path):
    try:
        async for message in websocket:
            print(f"Received message: {message}")
            response = f"{message}"
            await websocket.send(response)
    except websockets.exceptions.ConnectionClosedError:
        print("Connection closed by the client.")

start_server = websockets.serve(handle_connection, "localhost", 8765)

asyncio.get_event_loop().run_until_complete(start_server)
asyncio.get_event_loop().run_forever()