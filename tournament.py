import multiprocessing
import subprocess
import time
import sys
import argparse
import ataxx
import time
import os
import signal
from sprt import *

# Constants
GAME_MILLISECONDS = 10000
GAME_INCREMENT_MILLISECONDS = 100
GAME_RESULT_NORMAL = 0
GAME_RESULT_OUT_OF_TIME = 1
GAME_RESULT_ILLEGAL_MOVE = 2

# SPRT settings
alpha = 0.05
beta = 0.1
elo0 = 0
elo1 = 5
cutechess_sprt = True
lower = log(beta / (1 - alpha))
upper = log((1 - beta) / alpha)
RATING_INTERVAL = 20

class Engine:
    my_subprocess = None
    name = None
    milliseconds = None
    debug_file = None

    def __init__(self, exe, debug_file):
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

    def send(self, command):
        self.debug_file.write("SEND {}: {}\n".format(self.name, command))
        self.my_subprocess.stdin.write(command + "\n")
        self.my_subprocess.stdin.flush()

    def read_line(self):
        line = self.my_subprocess.stdout.readline().strip()
        if line != None and line != "":
            self.debug_file.write("RECV {}: {}\n".format(self.name, line))
        return line

def worker(process_id, exe1, exe2, shared, openings):
    assert(len(openings) == 880)
    assert(len(shared.keys()) == 8)
    import random

    board = None
    debug_file = open("debug/" + str(process_id) + ".txt", "w")
    eng1 = Engine(exe1, debug_file)
    eng2 = Engine(exe2, debug_file)
    eng_red = None
    eng_blue = None
    eng_to_play = None

    print("Initialized board", process_id)

    def send_go():
        nonlocal board
        nonlocal eng1
        nonlocal eng2
        nonlocal eng_red
        nonlocal eng_blue
        nonlocal eng_to_play
        assert board != None
        assert eng1 != None and eng2 != None
        assert eng_red != None and eng_blue != None
        assert eng_to_play != None

        command = "go"
        command += " rtime " + str(eng_red.milliseconds)
        command += " btime " + str(eng_blue.milliseconds)
        command += " rinc " + str(GAME_INCREMENT_MILLISECONDS)
        command += " binc " + str(GAME_INCREMENT_MILLISECONDS)

        eng_to_play.send(command)

    def play_game():
        nonlocal board
        nonlocal eng1
        nonlocal eng2
        nonlocal eng_red
        nonlocal eng_blue
        nonlocal eng_to_play
        nonlocal openings
        assert eng1 != None and eng2 != None

        # Prepare new game

        fen_opening = openings[random.randint(0, len(openings) - 1)].strip()
        board = ataxx.Board(fen_opening)

        my_fen = fen_opening.replace("x", "r").replace("o", "b").strip()
        eng1.send("position fen " + my_fen)
        eng2.send("position fen " + my_fen)

        # randomly decide engine color
        if random.randint(0,1) == 0:
            eng_red = eng1
            eng_blue = eng2
        else:
            eng_red = eng2
            eng_blue = eng1

        color = my_fen.split(" ")[-3].strip()
        assert color == "r" or color == "b"
        eng_to_play = eng_red if color == "r" else eng_blue

        eng1.milliseconds = eng2.milliseconds = GAME_MILLISECONDS

        assert board != None
        assert eng_red != None and eng_blue != None
        assert eng_to_play != None
        assert eng_red != eng_blue
        assert eng_to_play == eng_red or eng_to_play == eng_blue
        assert eng_to_play == eng1 or eng_to_play == eng2

        # Play out game
        while True:
            send_go()
            start_time = time.time()

            # wait for "bestmove" from engine
            while True:
                line = eng_to_play.read_line()
                if line.startswith("bestmove"):
                    break

            eng_to_play.milliseconds -= int((time.time() - start_time) * 1000)
            if eng_to_play.milliseconds <= 0:
                return GAME_RESULT_OUT_OF_TIME

            str_move = line.split(" ")[-1].strip()
            if not board.is_legal(ataxx.Move.from_san(str_move)):
                return GAME_RESULT_ILLEGAL_MOVE
            board.makemove(ataxx.Move.from_san(str_move))

            if board.gameover():
                return GAME_RESULT_NORMAL

            eng1.milliseconds += GAME_INCREMENT_MILLISECONDS
            eng2.milliseconds += GAME_INCREMENT_MILLISECONDS

            eng_to_play = eng_red if eng_to_play == eng_blue else eng_blue

            fen = board.get_fen().replace("x", "r").replace("o", "b").strip()
            eng1.send("position fen " + fen)
            eng2.send("position fen " + fen)

    def play_games():
        nonlocal board
        nonlocal eng1
        nonlocal eng2
        nonlocal eng_red
        nonlocal eng_blue
        nonlocal eng_to_play
        nonlocal shared

        # Main worker() loop, play games over and over
        while True:
            game_result_type = play_game()
            shared["games"].value += 1

            if game_result_type != GAME_RESULT_NORMAL:
                if eng_to_play == eng_red:
                    shared["w_blue"].value += 1
                    shared["l_red"].value += 1
                else:
                    assert eng_to_play == eng_blue
                    shared["l_blue"].value += 1
                    shared["w_red"].value += 1
                if eng_to_play == eng1:
                    shared["l"].value += 1
                else:
                    assert eng_to_play == eng2
                    shared["w"].value += 1
            else:
                str_result = board.result().strip()
                if str_result == "1-0":
                    shared["w_red"].value += 1
                    shared["l_blue"].value += 1
                    if eng1 == eng_red:
                        shared["w"].value += 1
                    else:
                        assert eng1 == eng_blue and eng2 == eng_red
                        shared["l"].value += 1
                elif str_result == "0-1":
                    shared["l_red"].value += 1
                    shared["w_blue"].value += 1
                    if eng1 == eng_blue:
                        shared["w"].value += 1
                    else:
                        assert eng1 == eng_red and eng2 == eng_blue
                        shared["l"].value += 1
                else:
                    shared["d"].value += 1

            games = shared["games"].value
            w = shared["w"].value
            l = shared["l"].value
            d = shared["d"].value
            w_red = shared["w_red"].value
            w_blue = shared["w_blue"].value
            l_red = shared["l_red"].value
            l_blue = shared["l_blue"].value

            print("(Board {}, {} vs {})".format(process_id, eng1.name, eng2.name), end="")
            print(" Total WLD {}-{}-{} ({})".format(w, l, d, games), end="")
            if game_result_type == GAME_RESULT_OUT_OF_TIME:
                if eng_to_play == eng1:
                    print(" Eng1 " + eng1.name + " out of time", end ="")
                else:
                    assert eng_to_play == eng2
                    print(" Eng2 " + eng2.name + " out of time", end ="")
            elif game_result_type == GAME_RESULT_ILLEGAL_MOVE:
                if eng_to_play == eng1:
                    print(" Eng1 " + eng1.name + " illegal move", end ="")
                else:
                    assert eng_to_play == eng2
                    print(" Eng2 " + eng2.name + " illegal move", end ="")
            print()

            if games % RATING_INTERVAL == 0:
                print("Red WL {}-{}".format(w_red, l_red))
                print("Blue WL {}-{}".format(w_blue, l_blue))
                llr = sprt(w, l, d, elo0, elo1, cutechess_sprt)
                e1, e2, e3 = elo_wld(w, l, d)
                print(f"ELO: {round(e2, 1)} +- {round((e3 - e1) / 2, 1)} [{round(e1, 1)}, {round(e3, 1)}]")
                print(f"LLR: {llr:.3} [{elo0}, {elo1}] ({lower:.3}, {upper:.3})")
                print("H1 accepted" if llr >= upper else ("H0 accepted" if llr <= lower else "Continue playing"))

    play_games()

if __name__ == "__main__":
    # Parse args
    parser = argparse.ArgumentParser(description="Run tournament between 2 Ataxx engines")
    parser.add_argument("--engine1", help="Engine 1 exe", type=str, required=True)
    parser.add_argument("--engine2", help="Engine 2 exe", type=str, required=True) 
    parser.add_argument("--concurrency", help="Concurrency", type=int, required=True) 
    args = parser.parse_args()
    print(args.engine1, "vs", args.engine2)
    print("Concurrency", args.concurrency)
    print()

    # Processes list (size = concurrency)
    processes = []

    def quit():
        print("Terminating engines...")
        # Wait for all processes to finish
        for process in processes:
            process.join()
        print("Engines terminated")
        exit(1)

    def handle_ctrl_c():
        print("Ctrl+C pressed")
        quit()

    # Register the CTRL+C signal handler
    signal.signal(signal.SIGINT, lambda signum, frame: handle_ctrl_c())

    # Load openings
    openings_file = open("openings_3ply.txt", "r")
    openings = openings_file.readlines()
    openings_file.close()
    assert len(openings) == 880

    # Create folder 'debug'
    if not os.path.exists("debug"):
        os.makedirs("debug")
    # Delete all files in 'debug'
    else:
        for filename in os.listdir("debug"):
            file_path = os.path.join("debug", filename)
        if os.path.isfile(file_path):
            os.unlink(file_path)

    # Create and run <concurrency> processes, each having 2 subprocesses
    with multiprocessing.Manager() as manager:
        shared = {
            'games': manager.Value('i', 0),
            'w': manager.Value('i', 0),
            'l': manager.Value('i', 0),
            'd': manager.Value('i', 0),
            'w_red': manager.Value('i', 0),
            'l_red': manager.Value('i', 0),
            'w_blue': manager.Value('i', 0),
            'l_blue': manager.Value('i', 0)
        }
        for i in range(args.concurrency):
            process = multiprocessing.Process(target=worker, args=(i+1, args.engine1, args.engine2, shared, openings))
            processes.append(process)
            process.start()
        for p in processes:
            p.join()