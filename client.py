import socket
from time import sleep

HOST = "127.0.0.1"
PORT = 1025


# request format
def generate_request(command: str):
    req_list = command.split()
    request_type = req_list[0]
    file_name = req_list[1]
    global HOST
    HOST = req_list[2]
    global PORT

    if len(req_list) < 4:
        PORT = 80
    else:
        PORT = req_list[3][req_list[3].find("(") + 1 : req_list[3].find(")")]

    request = f"{request_type} /{file_name} HTTP/1.1\r\nHost: {HOST}:{PORT}\r\n"
    return request, request_type, file_name

if __name__ == "__main__":

    # open inputs
    with open("commands.txt") as fp:
        while True:
            line = fp.readline()
            if line == "\n":
                continue
            if not line:
                break
            request, request_type, filename = generate_request(line.strip())
            print(request)
            # create socket connection
            with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
                s.connect((HOST, int(PORT)))
                # GET --> recieve the content and write it to a file
                if request_type == "GET":
                    request += "\r\n"
                    s.sendall(request.encode())
                    data = s.recv(10000)
                    data = data.split(b"\r\n\r\n")
                    msg_len = data[0].split(b": ")[-1]
                    content = data[1]
                    with open(filename, "wb") as f:
                        f.write(content)
                # POST --> read the file and send it to server then wait for the server's response
                elif request_type == "POST":
                    with open(filename, "rb") as f:
                        data = f.read()
                        request += f"Content-Length: {len(data)}\r\n\r\n"
                        to_be_sent = request.encode() + data
                        print(to_be_sent)
                        s.sendall(to_be_sent)
                        rcvd = s.recv(10000)
                        print(rcvd)
