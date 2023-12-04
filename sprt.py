from math import pow, sqrt, log, log10, copysign, pi
from dataclasses import dataclass


@dataclass
class Probability:
    win: float
    loss: float
    draw: float


@dataclass
class BayesElo:
    elo: float
    draw: float


def expected_score(x: float) -> float:
    return 1.0 / (1.0 + pow(10, -x / 400.0))


def adj_probs(b: BayesElo) -> Probability:
    win = expected_score(-b.draw + b.elo)
    loss = expected_score(-b.draw - b.elo)
    return Probability(win, loss, 1 - win - loss)


def scale(draw_elo: float) -> float:
    x = pow(10, -draw_elo / 400)
    return 4 * x / pow(1 + x, 2)


def sprt(
        wins: int,
        losses: int,
        draws: int,
        elo0: float,
        elo1: float,
        cutechess: bool = False
        ) -> float:
    if wins == 0 or losses == 0 or draws == 0:
        return 0.0

    total = wins + draws + losses

    probs = Probability(wins / total, losses / total, draws / total)

    draw_elo = 200 * log10((1 - 1 / probs.win) * (1 - 1 / probs.loss))

    # cutechess applies a draw elo based scaling
    s = 1
    if cutechess:
        s = scale(draw_elo)
        print(f"Adjusted Bounds: [{elo0 / s:.3}, {elo1 / s:.3}]")

    b0 = BayesElo(elo0 / s, draw_elo)
    b1 = BayesElo(elo1 / s, draw_elo)

    p0 = adj_probs(b0)
    p1 = adj_probs(b1)

    return wins * log(p1.win / p0.win) \
        + losses * log(p1.loss / p0.loss) \
        + draws * log(p1.draw / p0.draw)

def erf_inv(x):
    a = 8 * (pi - 3) / (3 * pi * (4 - pi))
    y = log(1 - x * x)
    z = 2 / (pi * a) + y / 2
    return copysign(sqrt(sqrt(z * z - y / a) - z), x)


def phi_inv(p):
    return sqrt(2)*erf_inv(2*p-1)


def elo(score: float) -> float:
    if score <= 0 or score >= 1:
        return 0.0
    return -400 * log10(1 / score - 1)


def elo_wld(wins, losses, draws):
    # win/loss/draw ratio
    N = wins + losses + draws
    if N == 0:
        return (0, 0, 0)

    p_w = float(wins) / N
    p_l = float(losses) / N
    p_d = float(draws) / N

    mu = p_w + p_d/2
    stdev = sqrt(p_w*(1-mu)**2 + p_l*(0-mu)**2 + p_d*(0.5-mu)**2) / sqrt(N)

    # 95% confidence interval for mu
    mu_min = mu + phi_inv(0.025) * stdev
    mu_max = mu + phi_inv(0.975) * stdev

    return (elo(mu_min), elo(mu), elo(mu_max))
