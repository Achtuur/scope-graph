import json

import matplotlib.pyplot as plt
import numpy as np
import plotlib as plb

BAD_SPEEDUP_THRESHOLD = 1.0
GOOD_SPEEDUP_THRESHOLD = 1.0001

BAD_SIZE_THRESHOLD = 10.0
GOOD_SIZE_THRESHOLD = 1.0

# found out by running with query sizes 1,2,4 and checking when speedup > 1
PERFORMANCE_BREAK_EVEN = {
    "fanout": {
        "sg_tree": 2, # all sizes
        "sg_linear": 2, #all sizes
        "sg_diamond": 1, #all sizes
        "sg_circle": 5, #all sizes
    },
    "linear": {
        "sg_tree": 2, # all sizes
        "sg_linear": 2, #all sizes
        "sg_diamond": 1, #all sizes
        "sg_circle": 4, #all sizes
    }
}

# 1,2,4 queries instead of 1,5,10
SMALL_BENCH_FILE = "./small-16-09-2025.json"
BENCH_FILE = "./q-16-09-2025.json"
# BENCH_FILE = "../scope-graph/output/benches/results.json"
NUM_QUERIES = ["1", "5", "10"]


def get_break_even(head: str, name: str) -> int:
    if "fan" in head:
        return PERFORMANCE_BREAK_EVEN["fanout"][name]
    else:
        return PERFORMANCE_BREAK_EVEN["linear"][name]

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

    def nano_float(self) -> float:
        return float(self.secs) * 1_000_000_000 + float(self.nanos)

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
    cache_size: int
    graph_size: int

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
        cache_size_estimate: float,
        cache_size: int,
        graph_size: int,
    ):
        self.time = time
        self.circle_check_time = circle_check_time
        self.cache_read_time = cache_read_time
        self.cache_store_time = cache_store_time
        self.cache_access_time = cache_read_time + cache_store_time
        self.edges_traversed = int(edges_traversed)
        self.nodes_visited = int(nodes_visited)
        self.cache_reads = int(cache_reads)
        self.cache_writes = int(cache_writes)
        self.cache_hits = int(cache_hits)
        self.cache_size_estimate = int(cache_size_estimate)
        self.cache_size = int(cache_size)
        self.graph_size = int(graph_size)

    def cache_frac(self) -> float:
        if self.graph_size == 0:
            return 0.0
        return self.cache_size / self.graph_size

    def cache_size_human_readable(self) -> str:
        # if self.cache_size < 1024:
        #     return f"{self.cache_size} B"
        # elif self.cache_size < 1024 * 1024:
        #     return f"{self.cache_size / 1024:.2f} KB"
        if self.cache_size < 1024 * 1024 * 1024:
            return f"{self.cache_size / (1024 * 1024):.2f} MB"
        else:
            return f"{self.cache_size / (1024 * 1024 * 1024):.2f} GB"

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
            cache_size=data["cache_size"],
            graph_size=data["graph_size"],
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
        self.num_queries = int(num_queries)
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

    def time(self, with_circle: bool) -> Time:
        if with_circle:
            return self.stats.time
        else:
            return self.stats.time_uncached() + self.stats.cache_access_time

    def circle_check_time(self, with_circle: bool) -> Time:
        if with_circle:
            return self.stats.circle_check_time
        else:
            return Time(0, 0)

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

    def pat_lbls(self):
        return [pattern_var_display_name(v) for v in self.pattern_var]


def pattern_var_display_name(var: str) -> str:
    tab = {
        "tree": "Tree",
        "linear": "Linear",
        "circle": "Circular",
        "diamond": "Diamond",
    }
    parts = var.split("-")
    name = parts[0]
    s1 = parts[1]
    if name == "diamond":
        s2 = parts[2]
        return f"{tab[name]} ($N = {s1}, M = {s2}$)"
    else:
        return f"{tab[name]} ($N = {s1}$)"

def pattern_var_table_entry(var: str) -> str:
    tab = {
        "tree": "Tree",
        "linear": "Linear",
        "circle": "Circular",
        "diamond": "Diamond",
    }
    parts = var.split("-")
    name = parts[0]
    s1 = parts[1]
    if name == "diamond":
        s2 = parts[2]
        entry = f"{tab[name]} \\\\ ($N = {s1}, M = {s2}$)"
    else:
        entry = f"{tab[name]} \\\\ ($N = {s1}$)"

    return f"\\multicolumn{{1}}{{p{{3cm}}}}{{\\centering {entry} }}"

def head_display_name(head: str) -> str:
    if "fanchain" in head:
        return "Fanout"
    elif "linear" in head:
        return "Linear"
    else:
        return head

class TableRow:
    entries: list[str]

    def __init__(self, entries: list[str]) -> None:
        self.entries = entries

    def append(self, entry: str) -> "TableRow":
        self.entries.append(entry)
        return self

    def extend(self, entries: list[str]) -> "TableRow":
        self.entries.extend(entries)
        return self

    def to_str(self) -> str:
        return " & ".join(self.entries) + " \\\\ \\hline\n"


class BenchResultVariation:
    var: VariationData
    data: dict[str, dict[str, list[BenchResult2]]]
    # base: dict[str, BenchResult2]
    # cached: list[BenchResult2]

    def __init__(self, var: VariationData, bench_results: list[BenchResult2]):
        self.var = var
        # dict where key is the var, value is list of bench results
        self.var_dict = {v: [] for v in var.pattern_var}
        # make dict out of this

        dct = {"base": {}, "cached": {}}

        for br in bench_results:
            if (
                br.name == var.name
                and br.head_type == var.head_type
                and br.pattern_var in var.pattern_var
            ):
                if dct[br.query_type].get(br.pattern_var) is None:
                    dct[br.query_type][br.pattern_var] = []
                dct[br.query_type][br.pattern_var].append(br)
                dct[br.query_type][br.pattern_var].sort(key=lambda x: x.num_queries)
        self.data = dct

    def base(self):
        return self.data["base"]

    def cached(self):
        return self.data["cached"]

    def base_var(self, var: str):
        return self.data["base"][var]

    def cached_var(self, var: str):
        return self.data["cached"][var]

    def to_table1_row(self) -> list[TableRow]:
        rows = []
        # base
        for var in self.var.pattern_var:
            exec_time_base = ""
            exec_time_cached = ""
            speedup = ""
            for b, c in zip(self.base_var(var), self.cached_var(var)):
                exec_time_base += f"{b.stats.time.as_num():.2f} / "
                cache_time = c.time("circle" in var)
                # cache_time = c.stats.time_uncached() + c.stats.cache_access_time
                exec_time_cached += f"{cache_time.as_num():.2f} / "
                speedup_num = b.stats.time.as_num() / cache_time.as_num()
                if speedup_num < BAD_SPEEDUP_THRESHOLD:
                    speedup += f"\\negative{{{speedup_num:.2f}x}} / "
                elif speedup_num > GOOD_SPEEDUP_THRESHOLD:
                    speedup += f"\\positive{{{speedup_num:.2f}x}} / "
                else:
                    speedup += f"{speedup_num:.2f}x / "

            exec_time_base = exec_time_base[:-3]  # remove trailing char
            exec_time_cached = exec_time_cached[:-3]  # remove trailing char
            speedup = speedup[:-3]  # remove trailing char
            break_even = str(get_break_even(self.var.head_type, self.var.name))
            row = TableRow(
                [
                    head_display_name(self.var.head_type),
                    pattern_var_display_name(var),
                    exec_time_base,
                    exec_time_cached,
                    speedup,
                    break_even
                ]
            )
            rows.append(row)

        return rows

    def to_table2_row(self) -> list[TableRow]:
        rows = []
        # cached only
        for var in self.var.pattern_var:
            with_circle = "circle" in var
            exec_time_cached = ""
            cache_time = ""
            cache_size = ""
            circle_time = ""
            c = self.cached_var(var)[-1]  # only use biggest numqueries
            total_time = c.time(with_circle).as_num()
            # for c in self.cached_var(var):
            exec_time_cached += (
                f"{100.0 * (c.stats.time_uncached().as_num() / total_time):.2f} / "
            )
            cache_time += (
                f"{100.0 * (c.stats.cache_access_time.as_num() / total_time):.2f} / "
            )
            circle_time += (
                f"{100.0 * (c.circle_check_time(with_circle).as_num() / total_time):.2f} / "
            )
            if c.stats.cache_frac() < GOOD_SIZE_THRESHOLD:
                cache_size += f"\\positive{{{c.stats.cache_frac():.2f}x ({c.stats.cache_size_human_readable()})}}"
            elif c.stats.cache_frac() > BAD_SIZE_THRESHOLD:
                cache_size += f"\\negative{{{c.stats.cache_frac():.2f}x ({c.stats.cache_size_human_readable()})}}"
            else:
                cache_size += f"{c.stats.cache_frac():.2f}x ({c.stats.cache_size_human_readable()})"
            exec_time_cached = exec_time_cached[:-3]  # remove trailing char
            cache_time = cache_time[:-3]
            circle_time = circle_time[:-3]
            row = TableRow(
                [
                    head_display_name(self.var.head_type),
                    pattern_var_display_name(var),
                    exec_time_cached,
                    cache_time,
                    circle_time,
                    cache_size,
                    "100\\%" # result must be same as base, otherwise we wouldn't have gotten the json
                ]
            )
            rows.append(row)

        return rows





def load_results2() -> list[BenchResult2]:
    with open(BENCH_FILE, "r") as f:
        data = json.load(f)
    return BenchResult2.from_json(data)


bench_data = load_results2()
print(f"Loaded {len(bench_data)} benchmark results.")
# for d in bench_data:
#     d.display()




class LabelStuff:
    lab: str
    color: plb.Color
    facecolor: plb.Color
    hatch: str | None

    def __init__(
        self,
        lab: str,
        color: plb.Color,
        facecolor: plb.Color | None = None,
        hatch: str | None = None,
    ) -> None:
        self.lab = lab
        self.color = color
        self.facecolor = facecolor or color
        self.hatch = hatch


class VarBarPlots:
    data: list[plb.Bar]
    # label, color, hatch
    labels: list[LabelStuff]

    def __init__(self, data: list[plb.Bar], labels: list[LabelStuff]) -> None:
        self.data = data
        self.labels = labels

    def plot(self, fig):
        fig.multiple_bars2(self.data)
        for l in self.labels:
            fig.bar(
                [0],
                [0],
                label=l.lab,
                color=l.color.rgb(17),
                hatch=l.hatch,
                edgecolor=l.color.rgb(17),
                facecolor=l.facecolor.rgb(17),
            )

    def max(self):
        return max([b.max() for b in self.data])


def get_data(
    bench_data: list[BenchResult2], var: VariationData, cycle_time: bool
) -> VarBarPlots:
    y = []

    cycle_time = "circle" in var.name

    def get_entry(num_queries: int, pattern_var: str, q_type: str) -> BenchResult2:
        matcher = BenchResult2(var.name, var.head_type, pattern_var, q_type, q, [])
        matcher.query_type = q_type
        try:
            return next(r for r in bench_data if r.eq_br(matcher))
        except StopIteration:
            raise Exception(f"Data not found for {matcher.to_str()}")

    labels = []
    labels.append(LabelStuff("$A_{cur}$", plb.Color.PURPLE))
    for q in NUM_QUERIES:
        uncached_time = []
        for pat in var.pattern_var:
            bench_res = get_entry(q, pat, "base")
            uncached_time.append(bench_res.stats.time_uncached().as_num())
        brightness = 15 + 2 * int(q)
        y.append(
            plb.Bar(
                [
                    plb.BarSegment(
                        np.array(uncached_time),
                        # label=f"{var.name}",
                        color=plb.Color.PURPLE.rgb(brightness),
                    ),
                ],
                f"{q}",
            )
        )

    labels.append(LabelStuff("$A_{mem}$ (graph)", plb.Color.GREEN))
    labels.append(
        LabelStuff("$A_{mem}$ (cache)", plb.Color.GREEN, plb.Color.ORANGE, "o")
    )
    if cycle_time:
        labels.append(
            LabelStuff("$A_{mem}$ (circle)", plb.Color.GREEN, plb.Color.BLUE, "/")
        )
    for q in NUM_QUERIES:
        uncached_time = []
        cache_time = []
        circle_time = []
        for pat in var.pattern_var:
            bench_res = get_entry(q, pat, "cached")
            uncached_time.append(bench_res.stats.time_uncached().as_num())
            cache_time.append(bench_res.stats.cache_access_time.as_num())
            circle_time.append(bench_res.stats.circle_check_time.as_num())
        brightness = 15 + 3 * int(q)
        color = plb.Color.GREEN.rgb(brightness)

        bars = [
            plb.BarSegment(
                np.array(uncached_time),
                # label=f"{var.name} uncached",
                color=color,
            ),
            plb.BarSegment(
                np.array(cache_time),
                # label=f"{var.name} cache",
                color=color,
                facecolor=plb.Color.ORANGE.rgb(brightness + 15),
                hatch="o",
            ),
        ]
        if cycle_time:
            bars.append(
                plb.BarSegment(
                    np.array(circle_time),
                    # label=f"{pat} circle check ({q} queries)",
                    color=color,
                    facecolor=plb.Color.BLUE.rgb(brightness),
                    hatch="/",
                )
            )

        y.append(plb.Bar(bars, f"{q}"))

    return VarBarPlots(y, labels)


fan_pat = "fanchain-25-10"
lin_pat = "linear-50"

VARIATIONS = {
    "sg_tree-fan": VariationData(
        "sg_tree",
        fan_pat,
        "Tree pattern with Fanout Head",
        ["tree-40", "tree-80", "tree-160"],
        fname="sg_tree-fan",
    ),
    "sg_tree-lin": VariationData(
        "sg_tree",
        lin_pat,
        "Tree pattern with Linear Head",
        ["tree-40", "tree-80", "tree-160"],
        fname="sg_tree-lin",
    ),
    "sg_linear-fan": VariationData(
        "sg_linear",
        fan_pat,
        "Linear pattern with Fanout Head",
        ["linear-20", "linear-40", "linear-80"],
        fname="sg_linear-fan",
    ),
    "sg_linear-lin": VariationData(
        "sg_linear",
        lin_pat,
        "Linear pattern with Linear Head",
        ["linear-20", "linear-40", "linear-80"],
        fname="sg_linear-lin",
    ),
    "sg_diamond-w-fan": VariationData(
        "sg_diamond",
        fan_pat,
        "Diamond pattern (varying width) with Fanout Head",
        ["diamond-4-1", "diamond-8-1", "diamond-16-1"],
        fname="sg_diamond-w-fan",
    ),
    "sg_diamond-w-lin": VariationData(
        "sg_diamond",
        lin_pat,
        "Diamond pattern (varying width) with Linear Head",
        ["diamond-4-1", "diamond-8-1", "diamond-16-1"],
        fname="sg_diamond-w-lin",
    ),
    "sg_diamond-h-fan": VariationData(
        "sg_diamond",
        fan_pat,
        "Diamond pattern (varying height) with Fanout Head",
        ["diamond-4-1", "diamond-4-2", "diamond-4-4"],
        fname="sg_diamond-h-fan",
    ),
    "sg_diamond-h-lin": VariationData(
        "sg_diamond",
        lin_pat,
        "Diamond pattern (varying height) with Linear Head",
        ["diamond-4-1", "diamond-4-2", "diamond-4-4"],
        fname="sg_diamond-h-lin",
    ),
    "sg_circle-fan": VariationData(
        "sg_circle",
        fan_pat,
        "Circle pattern with Fanout Head",
        ["circle-4", "circle-16", "circle-64"],
        fname="sg_circle-fan",
    ),
    "sg_circle-lin": VariationData(
        "sg_circle",
        lin_pat,
        "Circle pattern with Linear Head",
        ["circle-4", "circle-16", "circle-64"],
        fname="sg_circle-lin",
    ),
}


def plot_var(var: VariationData, save: bool = False):
    print(f"Plotting {var.name} ({var.head_type})...")
    fig = plb.SuperFigure.subplots()
    fig.set_default_plot_style()
    fig.set_size(plb.FigSize.MEDIUM)
    y = get_data(bench_data, var, False)
    y.plot(fig)

    max_y = y.max()
    fig.set_lim(None, (0.0, max_y * 1.25))

    fig.set_text(
        title=f"Execution Time for 1, 5 and 10 queries for {var.plot_title}",
        xlabel="Variation",
        ylabel="Execution time ($ms$)",
        legend_loc='upper left'
    )
    fig.set_xtick_label(var.pat_lbls())
    # fig.set_lim(None, (0, 4))

    if save:
        fig.save_figure(f"plots/{var.fname}", file_extension="png")
        fig.save_figure(f"plots/{var.fname}", file_extension="eps")


bench_vars = []
for v in VARIATIONS.values():
    var = BenchResultVariation(v, bench_data)
    bench_vars.append(var)


## table 1 : execution time for base and cache in same row
table1_rows = "".join([row.to_str() for v in bench_vars for row in v.to_table1_row()])
table1_str = f"""\\begin{{table}}[!ht]
\\tiny
\\caption{{Table showing performance results for \\algoOld{{}} compared to \\algoNew{{}}. Positive speedups are marked in green, while negative speedup is marked in red. The column on the right shows after how many queries \\algoNew{{}} achieves a speedup of at least 1.0x}}
\\label{{tab:algo-results}}
\\begin{{tabular}}{{ll|ccc|l}}
Head
& Pattern
& \\multicolumn{{1}}{{p{{3cm}}}}{{\\centering \\algoOld{{}} \\\\ 1/5/10 Queries \\\\ ($ms$)}}
& \\multicolumn{{1}}{{p{{3cm}}}}{{\\centering \\algoNew{{}} \\\\ 1/5/10 Queries \\\\ ($ms$)}}
& \\multicolumn{{1}}{{p{{3cm}}}}{{\\centering Speedup \\\\ 1/5/10 Queries}}
& Break Even
\\\\ \\toprule
{table1_rows}
\\end{{tabular}}
\\end{{table}}
"""

## table 2: cache only, comparing cache sizes
table2_rows = "".join([row.to_str() for v in bench_vars for row in v.to_table2_row()])
table2_str = f"""\\begin{{table}}[!ht]
\\tiny
\\caption{{Table showing performance results for \\algoNew{{}}. Very small caches are marked in green, while very large caches are marked in red.}}
\\label{{tab:cache-results}}
\\begin{{tabular}}{{ll|ccc|ll}}
Head
& Pattern
& \\multicolumn{{1}}{{p{{1.5cm}}}}{{\\centering Traversal Time \\\\ ($\\%$)}}
& \\multicolumn{{1}}{{p{{1.5cm}}}}{{\\centering Cache Time \\\\ ($\\%$)}}
& \\multicolumn{{1}}{{p{{1.5cm}}}}{{\\centering Circle Check \\\\ Time ($\\%$)}}
& \\multicolumn{{1}}{{p{{2cm}}}}{{\\centering Cache size \\\\ (compared to Scope Graph)}}
& \\multicolumn{{1}}{{p{{2cm}}}}{{\\centering Same result as \\\\ \\algoOld{{}}}}
\\\\ \\toprule
{table2_rows}
\\end{{tabular}}
\\end{{table}}
"""

with open("plots/comparison_table.tex", "w") as f:
    f.write(table1_str)

with open("plots/cache_result_table.tex", "w") as f:
    f.write(table2_str)


# plot_var(VARIATIONS["sg_linear-lin"], save=False)
for v in VARIATIONS.values():
    # if "fan" not in v.plot_title:
    plot_var(v, save=True)
    # plt.show()
    # break


# plt.show()
