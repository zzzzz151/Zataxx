from engine import Engine
from sprt import *
import ataxx
import time
import random

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

def worker(process_id, exe1, exe2, shared, tc_milliseconds, tc_increment_milliseconds, openings):
    print("Starting worker/board", process_id)

    assert(len(openings) >= 1)
    assert(len(shared.keys()) == 6)

    board = None # Our ataxx board, using the python ataxx library

    # Debug file for this worker's engines, a debug file is made for each worker (debug/1.txt, debug/2.txt, ...)
    debug_file = open("debug/" + str(process_id) + ".txt", "w")

    eng1 = Engine(exe1, debug_file) # First engine passed in program args
    eng2 = Engine(exe2, debug_file) # Second engine passed in program args
    eng_red = None     # Engine playing red, either eng1 or eng2
    eng_blue = None    # Engine playing blue, either eng1 or eng2
    eng_to_play = None # Engine to play current turn
    current_opening = 0

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
        command += " rinc " + str(tc_increment_milliseconds)
        command += " binc " + str(tc_increment_milliseconds)
        eng_to_play.send(command)

    # Setup a game and play it until its over, returning the result (see constants)
    def play_game():
        nonlocal board, eng1, eng2, eng_red, eng_blue, eng_to_play, current_opening
        assert eng1 != None and eng2 != None
        assert len(openings) >= 1

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
        eng1.milliseconds = eng2.milliseconds = tc_milliseconds

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

            # Add increment to engine that just played this turn
            eng_to_play.milliseconds += tc_increment_milliseconds

            # Switch sides: the other engine will play the next turn
            eng_to_play = eng_red if eng_to_play == eng_blue else eng_blue

    # Main loop, play games over and over
    while True:
        # Play a full game and get the result (see constants)
        game_result_type = play_game()

        # Update shared variables between the processes: wins, losses, draws, etc
        shared["games"].value += 1
        # If game over due to out of time or illegal move
        if game_result_type != GAME_RESULT_NORMAL:
            # if red lost
            if eng_to_play == eng_red: 
                shared["l_red"].value += 1
            # else blue lost
            else: 
                assert eng_to_play == eng_blue
                shared["w_red"].value += 1
            # if eng1 lost
            if eng_to_play == eng1: 
                shared["l"].value += 1
            # else eng1 won
            else:
                assert eng_to_play == eng2
                shared["w"].value += 1
        # Else game is over naturally
        else:
            str_result = board.result().strip()
            # if red won
            if str_result == "1-0": 
                shared["w_red"].value += 1
                # if eng1 won as red
                if eng1 == eng_red:
                    shared["w"].value += 1
                # else eng1 lost as blue
                else: 
                    assert eng1 == eng_blue and eng2 == eng_red
                    shared["l"].value += 1
            # else if blue won
            elif str_result == "0-1":
                shared["l_red"].value += 1
                # if eng1 won as blue
                if eng1 == eng_blue: 
                    shared["w"].value += 1
                # else eng1 lost as red
                else: 
                    assert eng1 == eng_red and eng2 == eng_blue
                    shared["l"].value += 1
            # else its a draw
            else: 
                shared["d"].value += 1

        # Grab the current wins, losses, draws from the shared data
        games = shared["games"].value
        w = shared["w"].value
        l = shared["l"].value
        d = shared["d"].value
        w_red = shared["w_red"].value
        l_red = shared["l_red"].value

        # Print new WDL
        print("({} vs {}, board {})".format(eng1.name, eng2.name, process_id), end="")
        print(" Total w-l-d {}-{}-{} ({})".format(w, l, d, games), end="")
        if game_result_type == GAME_RESULT_OUT_OF_TIME:
            print("", eng_to_play.name, "out of time", end="")
        elif game_result_type == GAME_RESULT_ILLEGAL_MOVE:
            print("", eng_to_play.name, "illegal move", end="")
        print()

        # Every RATING_INTERVAL games, print red WL, blue WL and current SPRT results
        if games % RATING_INTERVAL == 0:
            print("Red w-l {}-{} | Blue w-l {}-{}".format(w_red, l_red, l_red, w_red))
            llr = sprt(w, l, d, elo0, elo1, cutechess_sprt)
            e1, e2, e3 = elo_wld(w, l, d)
            print(f"ELO: {round(e2, 1)} +- {round((e3 - e1) / 2, 1)} [{round(e1, 1)}, {round(e3, 1)}]")
            print(f"LLR: {llr:.3} [{elo0}, {elo1}] ({lower:.3}, {upper:.3})")
            if llr >= upper:
                print("H1 accepted")
            elif llr <= lower:
                print("H0 accepted")