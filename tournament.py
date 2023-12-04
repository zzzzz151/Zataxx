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
from sprt import *

def handle_ctrl_c(subprocesses):
    print("Ctrl+C pressed. Terminating engines...")
    for subprocess in subprocesses:
        subprocess.terminate()
    print("Engines terminated.")
    exit(1)

def run_game(engine1, engine2):
    engine1.send_command_read_response("uainewgame")
    engine2.send_command_read_response("uainewgame")

    fen_opening = OPENINGS[random.randint(0, len(OPENINGS) - 1)].strip()
    board = ataxx.Board(fen_opening)
    fen_opening = fen_opening.replace("x", "r").replace("o", "b").strip()

    # randomly decide engine color, openings_3ply.txt is 3 plies so always blue to move
    if random.randint(0,1) == 0:
        engine1.color = RED
        engine2.color = BLUE
        engine_to_play = engine2
    else:
        engine1.color = BLUE
        engine2.color = RED
        engine_to_play = engine1

    engine1.milliseconds_left = engine2.milliseconds_left = GAME_MILLISECONDS

    while not board.gameover():
        fen = board.get_fen().replace("x", "r").replace("o", "b").strip()
        engine1.send_command_read_response("position fen " + fen)
        engine2.send_command_read_response("position fen " + fen)

        if engine1.color == RED:
            red_milliseconds_left = engine1.milliseconds_left
            blue_milliseconds_left = engine2.milliseconds_left
        else:
            red_milliseconds_left = engine2.milliseconds_left
            blue_milliseconds_left = engine1.milliseconds_left

        command = "go rtime " + str(red_milliseconds_left) + " btime " + str(blue_milliseconds_left)
        command += " rinc " + str(GAME_INCREMENT_MILLISECONDS) + " binc " + str(GAME_INCREMENT_MILLISECONDS)

        turn_start_time = time.time()
        str_move = engine_to_play.send_command_read_response(command).split(" ")[1]

        # subtract time spent this turn
        engine_to_play.milliseconds_left -= int((time.time() - turn_start_time) * 1000)

        if engine_to_play.milliseconds_left <= 0:
            return (ENGINE1_LOSS, ENGINE1_OUT_OF_TIME) if engine1 == engine_to_play else (ENGINE2_LOSS, ENGINE2_OUT_OF_TIME)

        board.makemove(ataxx.Move.from_san(str_move))
        engine_to_play = engine1 if engine_to_play == engine2 else engine2        
        engine1.milliseconds_left += GAME_INCREMENT_MILLISECONDS
        engine2.milliseconds_left += GAME_INCREMENT_MILLISECONDS

    str_result = board.result().strip()
    if str_result == "1-0":
        return ENGINE1_WIN if engine1.color == RED else ENGINE1_LOSS, GAME_RESULT_NORMAL
    elif str_result == "0-1":
        return ENGINE1_WIN if engine1.color == BLUE else ENGINE1_LOSS, GAME_RESULT_NORMAL
    return DRAW, GAME_RESULT_NORMAL

class Game:
    board = None
    engine1 = None
    engine2 = None
    name1 = "No name"
    name2 = "No name"
    engine_red = None
    engine_blue = None
    milliseconds_red = None
    milliseconds_blue = None
    GAME_MILLISECONDS = 10000
    GAME_INCREMENT_MILLISECONDS = 100

    def __init__(self, exe1, exe2):
        self.engine1 = subprocess.Popen(exe1, stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)
        self.engine2 = subprocess.Popen(exe2, stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)

        # Send "uai" and get name of each engine
        for eng in [self.engine1, self.engine2]:
            subprocess.stdin.write("uai\n")
            subprocess.stdin.flush()
            while True:
                line = subprocess.stdout.readline().strip()
                if line.startswith("id name"):
                    if eng == self.engine1:
                        self.name1 = line[7:-1].strip()
                    else:
                        self.name2 = line[7:-1].strip()
                elif line == "uaiok": 
                    break

    def swap_colors(self):
        temp = self.engine_red
        self.engine_red = self.engine_blue
        self.engine_blue = temp

    def terminate_engines(self):
        self.engine_red.terminate()
        self.engine_blue.terminate()

    def read_stdout(self, stdout):
        if self.engine1.stdout != stdout and self.engine2.stdout != stdout:
            return False

        eng = self.engine1 if self.engine1.stdout == stdout else self.engine2
        line = eng.stdout.readline().strip()

        if line.startswith("bestmove"):
            str_move = line.split(" ")[-1].strip()
            board.makemove(ataxx.Move.from_san(str_move))

            if board.gameover():
                self.onGameOver()
                return

            fen = board.get_fen()..replace("x", "r").replace("o", "b").strip()
            new_color = fen.split(" ")[-3]
            assert new_color == "r" or new_color == "b"
            color_that_played = "o" if new_color == "x" else "x"


    def onGameOver(self):
        str_result = board.result().strip()
        games += 1
        if str_result == "1-0":
            w_red += 1
            l_blue += 1
            if self.engine1 == self.engine_red:
                w += 1
            else:
                l += 1
        else if str_result == "0-1":
            w_blue += 1
            l_red += 1
            if self.engine1 == self.engine_red:
                l += 1
            else:
                w += 1
        else:
            d += 1

        assert w_red + w_blue == w + l
        assert w + l + d == games
        assert w_red + w_blue + l_red + l_blue == games - d

        print("(" + name1 + " vs " + name2 + ")", end="")
        print(" WLD", w, "-", l, "-", d, "(" + str(games) + ")", end ="")
        if game_result_type != GAME_RESULT_NORMAL:
            assert game_result_type == ENGINE1_OUT_OF_TIME or game_result_type == ENGINE2_OUT_OF_TIME
            print(" Engine 1 " + engine1.name if ENGINE1_OUT_OF_TIME else "Engine 2 " + engine2.name, end="")
            print(" out of time", end="")
        print(", red wins " + str(w_red) + ", blue wins " + str(w_blue))

        if games % RATING_INTERVAL == 0:
            llr = sprt(w, l, d, elo0, elo1, cutechess_sprt)
            e1, e2, e3 = elo_wld(w, l, d)
            e1 = int(e1)
            e2 = int(e2)
            e3 = int(e3)
            print(f"ELO: {e2} +- {(e3 - e1) / 2} [{e1}, {e3}]")
            print(f"LLR: {llr:.3} [{elo0:.3}, {elo1:.3}] ({lower:.3}, {upper:.3})")
            print("H1 accepted" if llr >= upper else ("H0 accepted" if llr <= lower else "Continue playing"))
            if llr >= upper or llr <= lower:
                for engine in engines:



# Parse args
parser = argparse.ArgumentParser(description="Run tournament between 2 Ataxx engines")
parser.add_argument("--engine1", help="Engine 1 exe", type=str, required=True)
parser.add_argument("--engine2", help="Engine 2 exe", type=str, required=True) 
parser.add_argument("--concurrency", help="Concurrency", type=int, required=True) 
args = parser.parse_args()
print("Concurrency", args.concurrency)

# Load openings
openingsFile = open("openings_3ply.txt", "r")
OPENINGS = openingsFile.readlines()
openingsFile.close()
assert len(OPENINGS) == 880

# Load games and engines
games = []
engines = []
for i in range(args.concurrency):
    games.append(Game(arg.engine1, arg.engine2))
    engines.append(games[-1].engine1)
    engines.append(games[-1].engine2)

# Register the CTRL+C signal handler
signal.signal(signal.SIGINT, lambda signum, frame: handle_ctrl_c(engines))

# Useful vars
w = l = d = games = w_red = w_blue = l_red = l_blue = 0

# SPRT settings
alpha = 0.05
beta = 0.1
elo0 = 0
elo1 = 5
cutechess_sprt = True
lower = log(beta / (1 - alpha))
upper = log((1 - beta) / alpha)
RATING_INTERVAL = 20

# Main loop
while True:
    # Use select to efficiently wait for data on stdout streams
    readable, _, _ = select.select([engine.stdout for engine in engines], [], [])
    for stdout in readable:
        for game in games:
            read_stdout(stdout)





    while True:
        game_result, game_result_type = run_game(engine1, engine2)
        games += 1
        if game_result == ENGINE1_WIN:
            w += 1
            if engine1.color == RED:
                assert engine2.color == BLUE
                w_red += 1
            else:
                assert engine1.color == BLUE and engine2.color == RED
                w_blue += 1
        elif game_result == ENGINE1_LOSS:
            l += 1
            if engine1.color == BLUE:
                assert engine2.color == RED
                w_red += 1
            else:
                assert engine1.color == RED and engine2.color == BLUE
                w_blue += 1
        else:
            assert game_result == DRAW
            d += 1

        assert w_red + w_blue == w + l
        assert w + l + d == games

        print("(" + engine1.name + " vs " + engine2.name + ")", end="")
        print(" WLD", w, "-", l, "-", d, "(" + str(games) + ")", end ="")
        if game_result_type != GAME_RESULT_NORMAL:
            assert game_result_type == ENGINE1_OUT_OF_TIME or game_result_type == ENGINE2_OUT_OF_TIME
            print(" Engine 1 " + engine1.name if ENGINE1_OUT_OF_TIME else "Engine 2 " + engine2.name, end="")
            print(" out of time", end="")
        print(", red wins " + str(w_red) + ", blue wins " + str(w_blue))

        if games % RATING_INTERVAL == 0:
            llr = sprt(w, l, d, elo0, elo1, cutechess_sprt)
            e1, e2, e3 = elo_wld(w, l, d)
            e1 = int(e1)
            e2 = int(e2)
            e3 = int(e3)
            print(f"ELO: {e2} +- {(e3 - e1) / 2} [{e1}, {e3}]")
            print(f"LLR: {llr:.3} [{elo0:.3}, {elo1:.3}] ({lower:.3}, {upper:.3})")
            print("H1 accepted" if llr >= upper else ("H0 accepted" if llr <= lower else "Continue playing"))
            if llr >= upper or llr <= lower:
                break

    print("Terminating engines...")
    engine1.terminate()
    engine2.terminate()
