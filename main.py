import socket
import mimetypes


def saveFile(fname, text):
    with open(fname, "wb") as f:
        f.write(text.split(b"\r\n\r\n", 1)[1])
    print("File saved as: " + fname)


def getData(fname):
    with open(fname, "rb") as f:
        d = f.read()
    ct = mimetypes.MimeTypes().guess_type(fileName)[0]
    return d, ct


with open("input.txt") as file:
    for line in file:
        tokens = line.split()
        httpMethod = tokens[0]
        fileName = tokens[1]
        serverIP = tokens[2]
        port = int(tokens[3]) if len(tokens) == 4 else 80

        clientSocket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        clientSocket.connect((serverIP, port))

        if httpMethod == "POST":
            data, contentType = getData(fileName)
            clientSocket.send("POST /{0} HTTP/1.1\r\nContent-Type: {2}\r\nContent-Length: {1}\r\n\r\n".format(fileName, len(data), contentType).encode() + data)
            response = clientSocket.recv(2048).decode()
            print(response)

        elif httpMethod == "GET":
            clientSocket.send("GET /{0} HTTP/1.1\r\nHost: {1}:{2}\r\n\r\n".format(fileName, serverIP, port).encode())
            response = clientSocket.recv(50000)
            saveFile(fileName, response)
            print(response.decode())
        clientSocket.close()
