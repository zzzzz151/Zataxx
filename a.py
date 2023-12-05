import ataxx
b = ataxx.Board("startpos")
print(b.is_legal(ataxx.Move.from_san("c7")))
print("hi")