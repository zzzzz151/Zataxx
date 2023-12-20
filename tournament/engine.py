import subprocess

class Engine:
    my_subprocess = None
    name = None
    milliseconds = None
    debug_file = None

    def __init__(self, exe, debug_file):
        # Start subprocess with .exe provided, 2*concurrency subprocesses are launched in total
        self.my_subprocess = subprocess.Popen(exe, stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)
        self.debug_file = debug_file

        # Init engine with "uai" and get engine name
        self.send("uai")
        while True:
            line = self.read_line()
            if line.startswith("id name"):
                self.name = line[7:].strip()
            if line == "uaiok":
                break

        assert self.name != None and self.name != ""
        #print("Initialized " + exe + " " + self.name)

    def __eq__(self, other):
        if other == None or other is None:
            return False
        assert isinstance(other, Engine)
        return self.my_subprocess == other.my_subprocess

    # Send command to engine
    def send(self, command):
        self.debug_file.write("SEND {}: {}\n".format(self.name, command))
        self.my_subprocess.stdin.write(command + "\n")
        self.my_subprocess.stdin.flush()

    # Read 1 line from engine and return it
    def read_line(self):
        line = self.my_subprocess.stdout.readline().strip()
        if line != None and line != "":
            self.debug_file.write("RECV {}: {}\n".format(self.name, line))
        return line