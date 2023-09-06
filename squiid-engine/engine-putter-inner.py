import pynng
import json

socket = pynng.Req0()
socket.dial("tcp://127.0.0.1:33242")

while True:
	payload = {
		'request_type': 'input',
		"payload": input('> ')
	}

	# payload = {
	# 	'request_type': 'configuration',
	# 	"payload": {
	# 		'action_type': 'list_sections',
	# 		'section': None,
	# 		'key': None
	# 	}
	# }

	socket.send(json.dumps(payload).encode('UTF-8'))
	print(socket.recv())
