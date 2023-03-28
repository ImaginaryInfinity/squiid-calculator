import zmq
import tty
import sys
import subprocess
import threading

context = zmq.Context()
socket = context.socket(zmq.REQ)
socket.connect("tcp://localhost:33242")


def rpnPut(ends=[], commands=[]):
    # Disable input buffering and echo
    tty.setcbreak(sys.stdin.fileno())
    char=""
    string=""
    while not char in ends and string not in commands:
        # Read a character
        char = sys.stdin.read(1)
        # Write the character
        sys.stdout.write(char)
        sys.stdout.flush()
        # Add to string unless it's an operator, then store it as the operator
        if not char in ends:
            string+=char
        else:
            op=char

    # Write newline after user is done
    sys.stdout.write("\n")
    # If string is a command, move it to the operator output
    if string in commands:
        op=string
        string=""
    return(string, op)

def backend():
    # Start backend
    subprocess.call(["target/debug/squid-engine"], stdout=subprocess.DEVNULL, stderr=subprocess.STDOUT)

def main():
    backendThread=threading.Thread(target=backend)
    backendThread.start()

    stack=[]
    operators=["\n", "+", "-", "/", "*", "_", "^"]
    commands=[
        "add",
        "subtract",
        "multiply",
        "divide",
        "power",
        "sqrt",
        "mod",
        "sin",
        "cos",
        "tan",
        "sec",
        "csc",
        "cot",
        "asin",
        "acos",
        "atan",
        "log",
        "logb",
        "ln",
        "abs",
        "eq",
        "gt",
        "lt",
        "gte",
        "lte",
        "invert",
        "drop",
        "swap",
        "dup",
        "rolldown",
        "rollup",
        "store",
        "clear",
        "quit"]
    while True:
        command_raw=rpnPut(ends=operators, commands=commands)
        if not command_raw[0]=="":
            socket.send_string(command_raw[0])
            print(socket.recv())
        match command_raw[1]:
            case '\n':
                if command_raw[0] =="":
                    command = "dup"
                else:
                    continue
            case '+':
                command = "add"
            case '-':
                command = "subtract"
            case '*':
                command = "multiply"
            case '/':
                command = "divide"
            case '_':
                command = "invert"
            case '^':
                command = "power"
            case default:
                command = command_raw[1]

        socket.send_string(command)
        print(socket.recv())

if __name__=="__main__":
    main()
