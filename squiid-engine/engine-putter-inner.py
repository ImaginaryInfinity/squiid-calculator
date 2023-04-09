import pynng

socket = pynng.Req0()
socket.dial("tcp://127.0.0.1:33242")

while True:
    socket.send(bytes(input('> '), encoding='UTF-8'))
    print(socket.recv())
