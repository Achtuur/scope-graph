import json

import matplotlib.pyplot as plt
import numpy as np

import plotlib as plb


class Time:
    secs: int
    nanos: int

    def __init__(self, secs, nanos):
        self.secs = secs
        self.nanos = nanos

    def from_json(data):
        return Time(secs=data["secs"], nanos=data["nanos"])

    def __float__(self):
        secs = float(self.secs) + float(self.nanos) / float(1_000_000_000)
        return secs * 1000.0

    def as_num(self):
        return float(self)


class BenchStats:
    time: Time
    edges_traversed: int
    nodes_visited: int
    cache_reads: int
    cache_writes: int
    cache_hits: int
    cache_size_estimate: int

    def __init__(
        self,
        time,
        edges_traversed,
        nodes_visited,
        cache_reads,
        cache_writes,
        cache_hits,
        cache_size_estimate,
    ):
        self.time = time
        self.edges_traversed = edges_traversed
        self.nodes_visited = nodes_visited
        self.cache_reads = cache_reads
        self.cache_writes = cache_writes
        self.cache_hits = cache_hits
        self.cache_size_estimate = cache_size_estimate

    def from_json(data):
        return BenchStats(
            time=Time.from_json(data["time"]),
            edges_traversed=data["edges_traversed"],
            nodes_visited=data["nodes_visited"],
            cache_reads=data["cache_reads"],
            cache_writes=data["cache_writes"],
            cache_hits=data["cache_hits"],
            cache_size_estimate=data["cache_size_estimate"],
        )


# map =
# [pattern-name -> head-type -> pattern_var -> base|cached -> num_queries -> BenchStats]
class BenchResult2:
    name: str
    head_type: str
    pattern_var: str
    query_type: str  # base | cached
    num_queries: int
    stats: BenchStats

    def __init__(self, name, head_type, pattern_var, query_type, num_queries, stats):
        self.name = name
        self.head_type = head_type
        self.pattern_var = pattern_var
        self.query_type = query_type
        self.num_queries = num_queries
        self.stats = stats

    def from_json(data):
        lst: list[BenchResult2] = []
        for name, val0 in data.items():
            for head_type, val1 in val0.items():
                for pattern_var, val2 in val1.items():
                    for query_type, val3 in val2.items():
                        for num_queries, stats_raw in val3.items():
                            stats = BenchStats.from_json(stats_raw)
                            res = BenchResult2(
                                name,
                                head_type,
                                pattern_var,
                                query_type,
                                num_queries,
                                stats,
                            )
                            lst.append(res)
        return lst

    def eq_br(self, other: "BenchResult2") -> bool:
        # return self.name == other.name and self.head_type == other.head_type and self.pattern_var == other.pattern_var and self.query_type == other.query_type and self.num_queries == other.num_queries
        return self.to_str() == other.to_str()

    def display(self):
        # print(self.name, self.head_type, self.pattern_var, self.query_type, self.num_queries)
        print(self.to_str())

    def to_str(self) -> str:
        return f"{self.name}::{self.head_type}::{self.pattern_var}::{self.query_type}::{self.num_queries}"


class BenchResult:
    name: str
    arg: str
    head: str
    stats: dict[int, BenchStats]

    def __init__(self, name, arg, head, stats):
        self.name = name
        self.arg = arg
        self.head = head
        self.stats = stats

    def from_json(name: str, data):
        data_map: dict[int, BenchStats] = {}
        for item in data:
            data_map[int(item["num_queries"])] = BenchStats.from_json(item["stats"])
        arg = data["arg"]
        head = data["head"]
        return BenchResult(name, arg, head, data_map)


class VariationData:
    name: str
    head_type: str
    pattern_var: list[str]
    plot_title: str
    fname: str

    def __init__(self, name, head_type, plot_title, pattern_var, fname=None):
        self.name = name
        self.head_type = head_type
        self.pattern_var = pattern_var
        self.plot_title = plot_title
        if fname is not None:
            self.fname = fname
        else:
            self.fname = f"bench-{self.name}-{self.head_type}"


BENCH_FILE_Q1 = "./q-21-07-2025.json"
BENCH_FILE = "../scope-graph/output/benches/results.json"


def load_results2() -> list[BenchResult2]:
    with open(BENCH_FILE, "r") as f:
        data = json.load(f)
    return BenchResult2.from_json(data)


bench_data = load_results2()
print(f"Loaded {len(bench_data)} benchmark results.")
for d in bench_data:
    d.display()


NUM_QUERIES = ["1", "2", "5"]


def get_data(bench_data: list[BenchResult2], var: VariationData) -> list[list[float]]:
    y = []

    def get_entry(num_queries: int, pattern_var: str, q_type: str):
        matcher = BenchResult2(var.name, var.head_type, pattern_var, q_type, q, [])
        matcher.query_type = q_type
        try:
            data = next(r for r in bench_data if r.eq_br(matcher))
            return data.stats.time.as_num()
        except StopIteration:
            raise Exception(f"Data not found for {matcher.to_str()}")

    for q in NUM_QUERIES:
        y.append([get_entry(q, pat, "base") for pat in var.pattern_var])
    for q in NUM_QUERIES:
        y.append([get_entry(q, pat, "cached") for pat in var.pattern_var])

    return y


fan_pat = "fanchain-25-10"
lin_pat = "linear-100"

VARIATIONS = {
    "sg_tree-fan": VariationData(
        "sg_tree",
        fan_pat,
        "Tree pattern with fan head",
        ["tree-40", "tree-80", "tree-160"],
        fname="sg_tree-fan",
    ),
    "sg_tree-lin": VariationData(
        "sg_tree",
        lin_pat,
        "Tree pattern with linear head",
        ["tree-40", "tree-80", "tree-160"],
        fname="sg_tree-lin",
    ),
    "sg_linear-fan": VariationData(
        "sg_linear",
        fan_pat,
        "Linear pattern with fan head",
        ["linear-40", "linear-80", "linear-160"],
        fname="sg_linear-fan",
    ),
    "sg_linear-lin": VariationData(
        "sg_linear",
        lin_pat,
        "Linear pattern with linear head",
        ["linear-40", "linear-80", "linear-160"],
        fname="sg_linear-lin",
    ),
    "sg_diamond-w-fan": VariationData(
        "sg_diamond",
        fan_pat,
        "Diamond pattern (varying width) with fan head",
        ["diamond-4-1", "diamond-8-1", "diamond-16-1"],
        fname="sg_diamond-w-fan",
    ),
    "sg_diamond-w-lin": VariationData(
        "sg_diamond",
        lin_pat,
        "Diamond pattern (varying width) with linear head",
        ["diamond-4-1", "diamond-8-1", "diamond-16-1"],
        fname="sg_diamond-w-lin",
    ),
    "sg_diamond-h-fan": VariationData(
        "sg_diamond",
        fan_pat,
        "Diamond pattern (varying height) with fan head",
        ["diamond-4-1", "diamond-4-2", "diamond-4-4"],
        fname="sg_diamond-h-fan",
    ),
    "sg_diamond-h-lin": VariationData(
        "sg_diamond",
        lin_pat,
        "Diamond pattern (varying height) with linear head",
        ["diamond-4-1", "diamond-4-2", "diamond-4-4"],
        fname="sg_diamond-h-lin",
    ),
    "sg_circle-fan": VariationData(
        "sg_circle",
        fan_pat,
        "Circle pattern with fan head",
        ["circle-4", "circle-16", "circle-64"],
        fname="sg_circle-fan",
    ),
    "sg_circle-lin": VariationData(
        "sg_circle",
        lin_pat,
        "Circle pattern with linear head",
        ["circle-4", "circle-16", "circle-64"],
        fname="sg_circle-lin",
    ),
}


def plot_var(var: VariationData, save: bool = False):
    print(f"Plotting {var.name} ({var.head_type})...")
    fig = plb.SuperFigure.subplots()
    fig.set_default_plot_style()
    fig.set_size(plb.FigSize.MEDIUM)
    y = get_data(bench_data, var)
    labels = []

    for q in NUM_QUERIES:
        labels.append(f"no cache ({q} queries)")
    for q in NUM_QUERIES:
        labels.append(f"cache ({q} queries)")

    colors = []
    for i in range(len(NUM_QUERIES)):
        colors.append(plb.Color.GREEN.rgb(15 * i))
    for i in range(len(NUM_QUERIES)):
        colors.append(plb.Color.PURPLE.rgb(15 * i))

    fig.multiple_bars(y, labels, colors)
    fig.set_text(
        title=var.plot_title, xlabel="Number of queries", ylabel="Execution time ($ms$)"
    )
    fig.set_xtick_label(var.pattern_var)
    # fig.set_lim(None, (0, 4))

    if save:
        fig.save_figure(f"plots/{var.fname}", file_extension="png")
        fig.save_figure(f"plots/{var.fname}", file_extension="eps")

    # plt.show()


# plot_var(VARIATIONS["sg_circle-fan"], save=True)

for v in VARIATIONS.values():
    plot_var(v, save=True)
