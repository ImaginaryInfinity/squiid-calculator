import zmq

context = zmq.Context()
socket = context.socket(zmq.REQ)
socket.connect("tcp://localhost:33242")

while True:
    socket.send_string(input())
    print(socket.recv())
