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
beta = 0.05
elo0 = 0
elo1 = 5
cutechess_sprt = True
RATING_INTERVAL = 20
lower = log(beta / (1 - alpha))
upper = log((1 - beta) / alpha)

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

def worker(process_id, exe1, exe2, shared, openings):
    import random
    assert(len(openings) >= 10)
    assert(len(shared.keys()) == 8)

    board = None # Our ataxx board, using the python ataxx library

    # Debug file for this worker's engines, a debug file is made for each worker (debug/1.txt, debug/2.txt, ...)
    debug_file = open("debug/" + str(process_id) + ".txt", "w")

    eng1 = Engine(exe1, debug_file) # First engine passed in program args
    eng2 = Engine(exe2, debug_file) # Second engine passed in program args
    eng_red = None     # Engine playing red, either eng1 or eng2
    eng_blue = None    # Engine playing blue, either eng1 or eng2
    eng_to_play = None # Engine to play current turn

    # Shuffle openings and initialize current opening index counter
    random.shuffle(openings)
    current_opening = 0

    print("Starting board", process_id)

    # Send "go rtime <rtime> btime <btime> rinc <rinc> binc <binc>" to engine to play current turn
    def send_go():
        nonlocal board, eng1, eng2, eng_red, eng_blue, eng_to_play
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

    # Setup a game and play it until its over, returning the result (see constants)
    def play_game():
        nonlocal board, eng1, eng2, eng_red, eng_blue, eng_to_play, current_opening
        assert eng1 != None and eng2 != None
        assert len(openings) >= 10

        eng1.send("uainewgame")
        eng2.send("uainewgame")

        # Get the next opening from the shuffled openings list
        fen_opening = openings[current_opening].strip()
        current_opening += 1
        if current_opening >= len(openings):
            current_opening = 0

        board = ataxx.Board(fen_opening)

        # Randomly decide engine color
        if random.randint(0,1) == 0:
            eng_red = eng1
            eng_blue = eng2
        else:
            eng_red = eng2
            eng_blue = eng1

        # Initialize eng_to_play
        color = fen_opening.split(" ")[-3].strip()
        assert color == "x" or color == "o"
        eng_to_play = eng_red if color == "x" else eng_blue

        # Reset each engine's time
        eng1.milliseconds = eng2.milliseconds = GAME_MILLISECONDS

        assert board != None
        assert eng_red != None and eng_blue != None and eng_to_play != None
        assert eng_red != eng_blue
        assert eng_to_play == eng_red or eng_to_play == eng_blue
        assert eng_to_play == eng1 or eng_to_play == eng2

        # Play out game until its over
        while True:
            # In the fen, replace "x" with "r" (red) and "o" with "b" (blue)
            my_fen = board.get_fen().replace("x", "r").replace("o", "b").strip()
            # Send "position fen <fen>" to both engines
            eng1.send("position fen " + my_fen)
            eng2.send("position fen " + my_fen)

            # Send go command and initialize time this turn started
            send_go()
            start_time = time.time()

            # Wait for "bestmove" from engine to play current turn
            while True:
                line = eng_to_play.read_line()
                if line.startswith("bestmove"):
                    break

            # Subtract the time the engine took this turn
            eng_to_play.milliseconds -= int((time.time() - start_time) * 1000)

            # Check if engine ran out of time
            if eng_to_play.milliseconds <= 0:
                return GAME_RESULT_OUT_OF_TIME

            # Get move as string from the "bestmove <move>" command
            str_move = line.split(" ")[-1].strip()

            # Check that the move the engine sent is legal
            if not board.is_legal(ataxx.Move.from_san(str_move)):
                return GAME_RESULT_ILLEGAL_MOVE

            # Apply the move the engine sent to our board
            board.makemove(ataxx.Move.from_san(str_move))

            # Check if game is over
            if board.gameover():
                return GAME_RESULT_NORMAL

            # Add increment time to both engines
            eng1.milliseconds += GAME_INCREMENT_MILLISECONDS
            eng2.milliseconds += GAME_INCREMENT_MILLISECONDS

            # Switch sides: the other engine will play the next turn
            eng_to_play = eng_red if eng_to_play == eng_blue else eng_blue

    # Main worker() loop, play games over and over
    def play_games():
        nonlocal board, eng1, eng2, eng_red, eng_blue, eng_to_play, shared

        while True:
            # PLay a full game and get the result (see constants)
            game_result_type = play_game()

            # Update shared variables between the processes: wins, losses, draws, etc
            shared["games"].value += 1
            # If game over due to out of time or illegal move
            if game_result_type != GAME_RESULT_NORMAL:
                if eng_to_play == eng_red: # red lost
                    shared["w_blue"].value += 1
                    shared["l_red"].value += 1
                else: # blue lost
                    assert eng_to_play == eng_blue
                    shared["l_blue"].value += 1
                    shared["w_red"].value += 1
                if eng_to_play == eng1: # eng1 lost
                    shared["l"].value += 1
                else: # eng1 won
                    assert eng_to_play == eng2
                    shared["w"].value += 1
            # Else game is over naturally
            else:
                str_result = board.result().strip()
                if str_result == "1-0": # red won
                    shared["w_red"].value += 1
                    shared["l_blue"].value += 1
                    if eng1 == eng_red: # eng1 won
                        shared["w"].value += 1
                    else: # eng1 lost
                        assert eng1 == eng_blue and eng2 == eng_red
                        shared["l"].value += 1
                elif str_result == "0-1": # blue won
                    shared["l_red"].value += 1
                    shared["w_blue"].value += 1
                    if eng1 == eng_blue: # eng1 won
                        shared["w"].value += 1
                    else: # eng1 lost
                        assert eng1 == eng_red and eng2 == eng_blue
                        shared["l"].value += 1
                else: # draw
                    shared["d"].value += 1

            # Grab the current wins, losses, draws from the shared data
            games = shared["games"].value
            w = shared["w"].value
            l = shared["l"].value
            d = shared["d"].value
            w_red = shared["w_red"].value
            w_blue = shared["w_blue"].value
            l_red = shared["l_red"].value
            l_blue = shared["l_blue"].value

            # Print new WDL
            print("(Board {}, {} vs {})".format(process_id, eng1.name, eng2.name), end="")
            print(" Total w-l-d {}-{}-{} ({})".format(w, l, d, games), end="")
            if game_result_type == GAME_RESULT_OUT_OF_TIME:
                print(eng_to_play.name, "out of time", end="")
            elif game_result_type == GAME_RESULT_ILLEGAL_MOVE:
                print(eng_to_play.name, "illegal move", end="")
            print()

            # Every RATING_INTERVAL games, print red WL, blue WL and current SPRT results
            if games % RATING_INTERVAL == 0:
                print("Red w-l {}-{} | Blue w-l {}-{}".format(w_red, l_red, w_blue, l_blue))
                llr = sprt(w, l, d, elo0, elo1, cutechess_sprt)
                e1, e2, e3 = elo_wld(w, l, d)
                print(f"ELO: {round(e2, 1)} +- {round((e3 - e1) / 2, 1)} [{round(e1, 1)}, {round(e3, 1)}]")
                print(f"LLR: {llr:.3} [{elo0}, {elo1}] ({lower:.3}, {upper:.3})")
                if llr >= upper:
                    print("H1 accepted")
                elif llr <= lower:
                    print("H0 accepted")

    # Start the main worker() loop
    play_games()

if __name__ == "__main__":
    import random

    # Parse args
    parser = argparse.ArgumentParser(description="Run tournament between 2 Ataxx engines")
    parser.add_argument("--engine1", help="Engine 1 exe", type=str, required=True)
    parser.add_argument("--engine2", help="Engine 2 exe", type=str, required=True) 
    parser.add_argument("--concurrency", help="Concurrency", type=int, required=True) 
    args = parser.parse_args()
    print(args.engine1, "vs", args.engine2)
    print("Concurrency", args.concurrency)
    print()

    # Processes/workers list (size = concurrency)
    processes = []

    def quit():
        print("Terminating engines...")
        # Wait for all processes/workers to finish
        for process in processes:
            process.join()
        print("Engines terminated")
        exit(1)

    def handle_ctrl_c():
        print("Ctrl+C pressed")
        quit()

    # Register the CTRL+C signal handler
    signal.signal(signal.SIGINT, lambda signum, frame: handle_ctrl_c())

    # Load openings from openings file
    openings_file = open("openings_3ply.txt", "r")
    openings = openings_file.readlines()
    openings_file.close()
    assert len(openings) >= 10

    # Create folder 'debug'
    if not os.path.exists("debug"):
        os.makedirs("debug")
    # Delete all files in 'debug' folder
    else:
        for filename in os.listdir("debug"):
            file_path = os.path.join("debug", filename)
            if os.path.isfile(file_path):
                os.unlink(file_path)

    # Create and run <concurrency> processes/workers, each having 2 subprocesses
    with multiprocessing.Manager() as manager:
        # Data shared between the <concurrency> processes/workers
        shared = {
            'games': manager.Value('i', 0),  # Total games finished
            'w': manager.Value('i', 0),      # Engine1 wins
            'l': manager.Value('i', 0),      # Engine1 losses
            'd': manager.Value('i', 0),      # Draws
            'w_red': manager.Value('i', 0),  # Red wins
            'l_red': manager.Value('i', 0),  # Red losses
            'w_blue': manager.Value('i', 0), # Blue wins
            'l_blue': manager.Value('i', 0)  # Blue losses
        }
        # Launch <concurrency> processes/workers, each will create 2 subprocesses (1 for each engine)
        for i in range(args.concurrency):
            process = multiprocessing.Process(target=worker, args=(i+1, args.engine1, args.engine2, shared, openings))
            processes.append(process)
            process.start()
        # Wait for processes/workers to end
        for p in processes:
            p.join()