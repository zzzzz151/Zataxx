from worker import worker
import argparse
import signal
import os
import multiprocessing

def split_list(input_list, n):
    sublist_size = len(input_list) // n
    remainder = len(input_list) % n
    start = 0
    result = []

    for i in range(n):
        end = start + sublist_size + (1 if i < remainder else 0)
        result.append(input_list[start:end])
        start = end

    return result

if __name__ == "__main__":
    import random

    # Parse args
    parser = argparse.ArgumentParser(description="Run tournament between 2 Ataxx engines")
    parser.add_argument("--engine1", help="Engine 1 exe", type=str, required=True)
    parser.add_argument("--engine2", help="Engine 2 exe", type=str, required=True) 
    parser.add_argument("--concurrency", help="Concurrency", type=int, required=True) 
    parser.add_argument("--tc", help="Time control", type=str, required=True) 
    parser.add_argument("--openings", help="Openings book .txt file", type=str, required=True) 
    args = parser.parse_args()

    print()
    print(args.engine1, "vs", args.engine2)
    print("Concurrency", args.concurrency)
    print("Time control", args.tc)
    print("Openings book", args.openings)
    print()

    # Parse time control
    tc_split = args.tc.split("+")
    assert(len(tc_split) <= 2 and len(tc_split) > 0)
    milliseconds = int(float(tc_split[0]) * 1000)
    increment_milliseconds = int(float(tc_split[1]) * 1000) if len(tc_split) == 2 else 0

    # Load openings from openings file
    openings_file = open(args.openings, "r")
    openings = openings_file.readlines()
    openings_file.close()
    assert len(openings) >= args.concurrency
    random.shuffle(openings)
    for i in range(len(openings)):
        openings[i] = openings[i].replace("r", "x").replace("b", "o")
    openings_split = split_list(openings, args.concurrency)

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

    # Create folder 'debug'
    if not os.path.exists("debug"):
        os.makedirs("debug")
    # Delete all files in 'debug' folder
    else:
        for filename in os.listdir("debug"):
            file_path = os.path.join("debug", filename)
            if os.path.isfile(file_path):
                os.unlink(file_path)

    # Create and run <concurrency> processes/workers, each having 2 subprocesses (1 for each engine)
    with multiprocessing.Manager() as manager:
        # Data shared between the <concurrency> processes/workers
        shared = {
            'games': manager.Value('i', 0),  # Total games finished
            'w': manager.Value('i', 0),      # Engine1 wins
            'l': manager.Value('i', 0),      # Engine1 losses
            'd': manager.Value('i', 0),      # Draws
            'w_red': manager.Value('i', 0),  # Red wins
            'l_red': manager.Value('i', 0),  # Red losses
        }
        # Launch <concurrency> processes/workers, each will create 2 subprocesses (1 for each engine)
        for i in range(args.concurrency):
            worker_args = (i+1, args.engine1, args.engine2, shared, milliseconds, increment_milliseconds, openings_split[i])
            process = multiprocessing.Process(target=worker, args=worker_args)
            processes.append(process)
            process.start()
        # Wait for processes/workers to end
        for p in processes:
            p.join()