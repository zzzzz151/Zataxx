import multiprocessing
import subprocess
import time
import sys
import argparse
import select
import ataxx
import random
import time
import signal
import selectors
from sprt import *

# Constants
GAME_MILLISECONDS = 10000
GAME_INCREMENT_MILLISECONDS = 100
GAME_RESULT_NORMAL = 0
GAME_RESULT_OUT_OF_TIME = 1

# Lists for engines and games
engines = []
games = []

# Useful vars
w = l = d = games_finished = w_red = w_blue = l_red = l_blue = 0

# SPRT settings
alpha = 0.05
beta = 0.1
elo0 = 0
elo1 = 5
cutechess_sprt = True
lower = log(beta / (1 - alpha))
upper = log((1 - beta) / alpha)
RATING_INTERVAL = 10

def quit():
    print("Terminating engines...")
    for engine in engines:
        engine.subprocess.terminate()
    print("Engines terminated")
    exit(1)

def handle_ctrl_c():
    print("Ctrl+C pressed")
    quit()

# Register the CTRL+C signal handler
signal.signal(signal.SIGINT, lambda signum, frame: handle_ctrl_c())

# Load openings
openingsFile = open("openings_3ply.txt", "r")
OPENINGS = openingsFile.readlines()
openingsFile.close()
assert len(OPENINGS) == 880

class Engine:
    subprocess = None
    name = None
    milliseconds = None

    def __init__(self, exe):
        self.subprocess = subprocess.Popen(exe, stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)
        assert self.subprocess != None

        # Init engine with "uai" and get engine name
        self.send("uai")
        while True:
            line = self.subprocess.stdout.readline().strip()
            if line.startswith("id name"):
                self.name = line[7:-1].strip()
            elif line == "uaiok": 
                break

        assert self.name != None and self.name != ""

    def __eq__(self, other):
        if other == None or other is None:
            return False
        assert isinstance(other, Engine)
        return self.subprocess == other.subprocess

    def send(self, command):
        self.subprocess.stdin.write(command + "\n")
        self.subprocess.stdin.flush()

class Game:
    id = None
    board = None
    eng1 = None
    eng2 = None
    eng_red = None
    eng_blue = None
    eng_to_play = None
    saved_time = None

    def __init__(self, id, eng1, eng2):
        self.id = id
        self.eng1 = eng1
        self.eng2 = eng2
        assert self.eng1 != None and self.eng2 != None

    def swap_colors(self):
        temp = self.eng_red
        self.eng_red = self.eng_blue
        self.eng_blue = temp

    def swap_eng_to_play(self):
        assert self.eng1 != None and self.eng2 != None
        assert self.eng_red != None and self.eng_blue != None
        assert self.eng_to_play != None

        self.eng_to_play = self.eng_red if self.eng_to_play == self.eng_blue else self.eng_blue

    def send_go(self):
        assert self.board != None
        assert self.eng1 != None and self.eng2 != None
        assert self.eng_red != None and self.eng_blue != None
        assert self.eng_to_play != None

        command = "go"
        command += " rtime " + str(eng_red.milliseconds)
        command += " btime " + str(eng_blue.milliseconds)
        command += " rinc " + str(GAME_INCREMENT_MILLISECONDS)
        command += " binc " + str(GAME_INCREMENT_MILLISECONDS)

        self.saved_time = time.time()
        self.eng_to_play.send(command)

    def start_new_game(self):
        assert self.eng1 != None and self.eng2 != None

        fen_opening = OPENINGS[random.randint(0, len(OPENINGS) - 1)].strip()
        board = ataxx.Board(fen_opening)

        my_fen = fen_opening.replace("x", "r").replace("o", "b").strip()
        self.eng1.send("position fen " + my_fen)
        self.eng2.send("position fen " + my_fen)

        # randomly decide engine color
        if random.randint(0,1) == 0:
            self.eng_red = self.eng1
            self.eng_blue = self.eng2
        else:
            self.eng_red = self.eng2
            self.eng_blue = self.eng1

        color = my_fen.split(" ").strip()[-3]
        assert color == "r" or color == "o"
        self.eng_to_play = self.eng_red if color == "r" else self.eng_blue

        self.eng1.milliseconds = self.eng2.milliseconds = GAME_MILLISECONDS

    def read_stdout(self, stdout):
        assert self.board != None
        assert self.eng1 != None and self.eng2 != None
        assert self.eng_red != None and self.eng_blue != None
        assert self.eng_to_play != None

        if stdout == None or stdout.strip() == "":
            return 

        if self.eng1.subprocess.stdout != stdout and self.eng2.subprocess.stdout != stdout:
            return 

        assert self.eng_to_play.subprocess.stdout == stdout
        line = self.eng_to_play.subprocess.stdout.readline().strip()

        if line.startswith("bestmove"):
            str_move = line.split(" ")[-1].strip()
            board.makemove(ataxx.Move.from_san(str_move))

            self.eng_to_play.milliseconds -= int((time.time() - self.saved_time) * 1000)
            if self.eng_to_play.milliseconds <= 0:
                self.on_game_over(GAME_RESULT_OUT_OF_TIME)

            if board.gameover():
                self.on_game_over(GAME_RESULT_NORMAL)
                return

            self.eng1.milliseconds += GAME_INCREMENT_MILLISECONDS
            self.eng2.milliseconds += GAME_INCREMENT_MILLISECONDS

            fen = self.board.get_fen().replace("x", "r").replace("o", "b").strip()
            self.eng1.send("position fen " + fen)
            self.eng2.send("position fen " + fen)
            self.swap_eng_to_play()
            self.send_go()

    def on_game_over(self, game_result_type):
        str_result = board.result().strip()
        games_finished += 1
        if str_result == "1-0" or (game_result_type == GAME_RESULT_OUT_OF_TIME and self.eng_to_play == self.engine_blue):
            w_red += 1
            l_blue += 1
            if self.eng1 == self.eng_red:
                w += 1
            else:
                l += 1
        elif str_result == "0-1" or (game_result_type == GAME_RESULT_OUT_OF_TIME and self.eng_to_play == self.engine_red):
            w_blue += 1
            l_red += 1
            if self.eng1 == self.eng_red:
                l += 1
            else:
                w += 1
        else:
            d += 1

        assert w_red + w_blue == w + l
        assert w + l + d == games_finsihed
        assert w_red + w_blue + l_red + l_blue == games_finished - d

        print("(Board " + str(self.id) + ")", end="")
        print("(" + self.eng1.name + " vs " + self.eng2.name + ")", end="")
        print(" WLD", w, "-", l, "-", d, "(" + str(games_finished) + ")", end ="")
        if game_result_type != GAME_RESULT_NORMAL:
            assert game_result_type == ENGINE1_OUT_OF_TIME or game_result_type == ENGINE2_OUT_OF_TIME
            print(" Engine 1 " + engine1.name if ENGINE1_OUT_OF_TIME else "Engine 2 " + engine2.name, end="")
            print(" out of time", end="")
        print(", red wins " + str(w_red) + ", blue wins " + str(w_blue))

        if games_finished % RATING_INTERVAL == 0:
            llr = sprt(w, l, d, elo0, elo1, cutechess_sprt)
            e1, e2, e3 = elo_wld(w, l, d)
            e1 = int(e1)
            e2 = int(e2)
            e3 = int(e3)
            print(f"ELO: {e2} +- {(e3 - e1) / 2} [{e1}, {e3}]")
            print(f"LLR: {llr:.3} [{elo0:.3}, {elo1:.3}] ({lower:.3}, {upper:.3})")
            print("H1 accepted" if llr >= upper else ("H0 accepted" if llr <= lower else "Continue playing"))
            if llr >= upper or llr <= lower:
                quit()

        self.start_new_game()

# Parse args
parser = argparse.ArgumentParser(description="Run tournament between 2 Ataxx engines")
parser.add_argument("--engine1", help="Engine 1 exe", type=str, required=True)
parser.add_argument("--engine2", help="Engine 2 exe", type=str, required=True) 
parser.add_argument("--concurrency", help="Concurrency", type=int, required=True) 
args = parser.parse_args()
print("Concurrency", args.concurrency)
print(args.engine1, "vs", args.engine2)

# Create a selector
selector = selectors.DefaultSelector()

# Load engines and games
for i in range(args.concurrency):
    eng1 = Engine(args.engine1)
    eng2 = Engine(args.engine2)
    engines.append(eng1)
    engines.append(eng2)
    games.append(Game(i+1, eng1, eng2))

subprocess_list = [engine.subprocess for engine in engines]

# Create a dictionary to store the subprocesses' output buffers
output_buffers = {process.stdout: [] for process in subprocess_list}

# Create a dictionary to store the subprocesses' input buffers
input_buffers = {process.stdin: [] for process in subprocess_list}

# Main loop
while any(process.poll() is None for process in subprocess_list):
    # Check for subprocesses with available data to read
    readable, _, _ = select.select(output_buffers.keys(), [], [], 0.1)

    # Read from readable subprocesses
    for stream in readable:
        process = next(process for process, stdout in output_buffers.items() if stdout is stream)
        line = stream.readline().strip()
        if line:
            print(f"Received from process {process.pid}: {line}")


quit()
