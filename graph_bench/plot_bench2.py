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

    def __add__(self, other: "Time") -> "Time":
        secs = self.secs + other.secs
        nanos = self.nanos + other.nanos
        if nanos >= 1_000_000_000:
            secs += 1
            nanos -= 1_000_000_000
        return Time(secs, nanos)

    def __sub__(self, other: "Time") -> "Time":
        secs = self.secs - other.secs
        nanos = self.nanos - other.nanos
        if nanos < 0:
            secs -= 1
            nanos += 1_000_000_000
        return Time(secs, nanos)


class BenchStats:
    time: Time
    circle_check_time: Time
    no_circle_check_time: Time
    cache_read_time: Time
    cache_store_time: Time
    cache_access_time: Time
    edges_traversed: int
    nodes_visited: int
    cache_reads: int
    cache_writes: int
    cache_hits: int
    cache_size_estimate: int

    def __init__(
        self,
        time: Time,
        circle_check_time: Time,
        cache_read_time: Time,
        cache_store_time: Time,
        edges_traversed: int,
        nodes_visited: int,
        cache_reads: int,
        cache_writes: int,
        cache_hits: int,
        cache_size_estimate: int,
    ):
        self.time = time
        self.circle_check_time = circle_check_time
        self.cache_read_time = cache_read_time
        self.cache_store_time = cache_store_time
        self.cache_access_time = cache_read_time + cache_store_time
        self.edges_traversed = edges_traversed
        self.nodes_visited = nodes_visited
        self.cache_reads = cache_reads
        self.cache_writes = cache_writes
        self.cache_hits = cache_hits
        self.cache_size_estimate = cache_size_estimate


    def time_uncached(self) -> Time:
        return self.time - self.cache_access_time - self.circle_check_time

    def from_json(data):
        return BenchStats(
            time=Time.from_json(data["time"]),
            circle_check_time=Time.from_json(data["circle_check_time"]),
            cache_read_time=Time.from_json(data["cache_read_time"]),
            cache_store_time=Time.from_json(data["cache_store_time"]),
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


NUM_QUERIES = ["1", "5", "10"]


def get_data(bench_data: list[BenchResult2], var: VariationData) -> list[list[plb.Bar]]:
    y = []

    def get_entry(num_queries: int, pattern_var: str, q_type: str) -> BenchResult2:
        matcher = BenchResult2(var.name, var.head_type, pattern_var, q_type, q, [])
        matcher.query_type = q_type
        try:
            return next(r for r in bench_data if r.eq_br(matcher))
        except StopIteration:
            raise Exception(f"Data not found for {matcher.to_str()}")

    for q in NUM_QUERIES:
        uncached_time = []
        for pat in var.pattern_var:
            bench_res =  get_entry(q, pat, "base")
            uncached_time.append(bench_res.stats.time_uncached().as_num())
        brightness = 15 + 2 * int(q)
        y.append(plb.Bar([
            plb.BarSegment(
                np.array(uncached_time),
                label=f"{var.name} ({q} queries)",
                color=plb.Color.PURPLE.rgb(brightness)
            ),
        ]))

    for q in NUM_QUERIES:
        uncached_time = []
        cache_time = []
        circle_time = []
        for pat in var.pattern_var:
            bench_res =  get_entry(q, pat, "cached")
            uncached_time.append(bench_res.stats.time_uncached().as_num())
            cache_time.append(bench_res.stats.cache_access_time.as_num())
            circle_time.append(bench_res.stats.circle_check_time.as_num())
        brightness = 15 + 3 * int(q)
        color = plb.Color.GREEN.rgb(brightness)
        y.append(plb.Bar([
            plb.BarSegment(
                np.array(uncached_time),
                label=f"{var.name} uncached ({q} queries)",
                color=color,
            ),
            plb.BarSegment(
                np.array(cache_time),
                label=f"{var.name} cache ({q} queries)",
                color=color,
                facecolor=plb.Color.ORANGE.rgb(brightness + 15),
                hatch='o',
            ),
            plb.BarSegment(
                np.array(circle_time),
                label=f"{pat} circle check ({q} queries)",
                color=color,
                facecolor=plb.Color.BLUE.rgb(brightness),
                hatch='/',
            ),
        ]))


    return y


fan_pat = "fanchain-15-10"
lin_pat = "linear-25"

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
        ["linear-20", "linear-40", "linear-80"],
        fname="sg_linear-fan",
    ),
    "sg_linear-lin": VariationData(
        "sg_linear",
        lin_pat,
        "Linear pattern with linear head",
        ["linear-20", "linear-40", "linear-80"],
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

    # fig.multiple_bars(y, labels, colors)
    fig.multiple_bars2(y)
    fig.set_text(
        title=var.plot_title, xlabel="Number of queries", ylabel="Execution time ($ms$)"
    )
    fig.set_xtick_label(var.pattern_var)
    # fig.set_lim(None, (0, 4))

    if save:
        fig.save_figure(f"plots/{var.fname}", file_extension="png")
        fig.save_figure(f"plots/{var.fname}", file_extension="eps")



# plot_var(VARIATIONS["sg_linear-lin"], save=False)
for v in VARIATIONS.values():
    if "fan" not in v.plot_title:
        plot_var(v, save=False)


# bar = [
#     plb.Bar([
#         plb.BarSegment(np.array([1, 2, 3]), label="Base", color=plb.Color.RED.rgb(15)),
#         plb.BarSegment(np.array([4, 5, 6]), label="Cached", color=plb.Color.GREEN.rgb(15))
#     ]),
#     plb.Bar([
#         plb.BarSegment(np.array([1, 1, 1]), label="Base2", color=plb.Color.BLUE.rgb(15)),
#         plb.BarSegment(np.array([2, 4, 6]), label="Cached2", color=plb.Color.PURPLE.rgb(15))
#     ])
# ]

# fig = plb.SuperFigure.subplots()
# fig.set_default_plot_style()
# fig.multiple_bars2(bar, width=0.1, bar_offset=0.01)
# fig.set_xtick_label(["Bar1", "Bar2", "bar3"])
# fig.set_text(
#     title="test", xlabel="Number of queries", ylabel="Execution time ($ms$)"
# )


plt.show()

