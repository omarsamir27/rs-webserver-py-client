import socket
import os
from json import dump, load
from shutil import copyfile
from time import sleep, time

HOST = "127.0.0.1"
PORT = 1025

cache = {}
try:
    manifest = open("manifest.json", "r")
    cache = load(manifest)
    manifest.close()
except:
    pass

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


def recv_timeout(socket, timeout=2):
    # make socket non blocking
    socket.setblocking(0)

    # total data partwise in an array
    total_data = [];
    data = '';

    # beginning time
    begin = time()
    while True:
        # if you got some data, then break after timeout
        if total_data and time() - begin > timeout:
            break

        # if you got no data at all, wait a little longer, twice the timeout
        elif time() - begin > timeout:
            break

        # recv something
        try:
            data = socket.recv(4096)
            if data:
                total_data.append(data)
                # change the beginning time for measurement
                begin = time()
            else:
                break
        except:
            pass

    # join all parts to make final string
    return b''.join(total_data)


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
            # create socket connection
            with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
                s.connect((HOST, int(PORT)))
                # GET --> recieve the content and write it to a file
                if request_type == "GET":

                    # conditional GET
                    if filename in cache:
                        found_path = cache[filename]
                        copyfile(os.path.join(found_path, filename), filename)
                        print(f"[FILE EXISTS] {filename} already exists in cache.")
                        continue

                    request += "\r\n"
                    s.sendall(request.encode())
                    data = recv_timeout(s, 0.5)
                    data = data.split(b"\r\n\r\n")
                    msg_len = data[0].split(b": ")[-1]
                    content = data[1]
                    print(data[0].decode("utf-8"))
                    with open(filename, "wb") as f:
                        f.write(content)
                    copyfile(filename, os.path.join(os.getcwd() , "cache", filename))
                    cache[filename] = os.path.join(os.getcwd(), "cache")

                # POST --> read the file and send it to server then wait for the server's response
                elif request_type == "POST":
                    with open(filename, "rb") as f:
                        data = f.read()
                        request += f"Content-Length: {len(data)}\r\n\r\n"
                        to_be_sent = request.encode() + data
                        s.sendall(to_be_sent)
                        rcvd = recv_timeout(s, 0.5)
                        print(rcvd)

    with open("manifest.json", "w") as manifest:
        dump(cache, manifest)
