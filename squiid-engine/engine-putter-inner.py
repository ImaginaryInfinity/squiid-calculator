import pynng
import json

socket = pynng.Req0()
socket.dial("tcp://127.0.0.1:33242")

while True:
	payload = {
		"payload": input('> ')
	}

	socket.send(json.dumps(payload).encode('UTF-8'))
	print(socket.recv())
