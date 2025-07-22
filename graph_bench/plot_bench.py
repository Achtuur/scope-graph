import json

import matplotlib.pyplot as plt
import numpy as np
import plotlib as plb


class ConfidenceInterval:
    level: float
    lower_bound: float
    upper_bound: float
    def __init__(self, level, lower_bound, upper_bound):
        self.level = level
        self.lower_bound = lower_bound
        self.upper_bound = upper_bound

    def div(self, other):
        return ConfidenceInterval(
            self.level,
            self.lower_bound / other.lower_bound,
            self.upper_bound / other.upper_bound
        )


# units are all ns
class Estimate:
    confidence_interval: ConfidenceInterval
    point_estimate: float
    standard_error: float
    def __init__(self, confidence_interval, point_estimate, standard_error):
        self.confidence_interval = confidence_interval
        self.point_estimate = point_estimate
        self.standard_error = standard_error

    def from_json(data):
        confidence_interval = ConfidenceInterval(
            level=data['confidence_interval']['confidence_level'],
            lower_bound=data['confidence_interval']['lower_bound'],
            upper_bound=data['confidence_interval']['upper_bound']
        )
        return Estimate(
            confidence_interval=confidence_interval,
            point_estimate=data['point_estimate'],
            standard_error=data['standard_error']
        )

    def div(self, other):
        return Estimate(
            self.confidence_interval.div(other.confidence_interval),
            self.point_estimate / other.point_estimate,
            self.standard_error / other.standard_error
        )

class BenchResult:
    mean: Estimate
    median: Estimate
    median_abs_dev: Estimate
    slope: Estimate
    std_dev: Estimate

    def __init__(self, mean, median, median_abs_dev, slope, std_dev):
        self.mean = mean
        self.median = median
        self.median_abs_dev = median_abs_dev
        self.slope = slope
        self.std_dev = std_dev

    def from_file(path: str):
        with open(path, 'r') as f:
            data = json.load(f)
        return BenchResult(
            mean=Estimate.from_json(data['mean']),
            median=Estimate.from_json(data['median']),
            median_abs_dev=Estimate.from_json(data['median_abs_dev']),
            slope=Estimate.from_json(data['slope']),
            std_dev=Estimate.from_json(data['std_dev'])
        )

    def div(self, other):
        return BenchResult(
            self.mean.div(other.mean),
            self.median.div(other.median),
            self.median_abs_dev.div(other.median_abs_dev),
            self.slope.div(other.slope),
            self.std_dev.div(other.std_dev)
        )



def load_estimates(name: str) -> tuple[list[BenchResult], list[BenchResult]]:
    paths = [
        f"../target/criterion/{name}/{name}_1_32/base/estimates.json",
        f"../target/criterion/{name}/{name}_2_32/base/estimates.json",
        f"../target/criterion/{name}/{name}_5_32/base/estimates.json",
    ]
    paths_cached = [
        f"../target/criterion/{name}/{name}_cached_1_32/base/estimates.json",
        f"../target/criterion/{name}/{name}_cached_2_32/base/estimates.json",
        f"../target/criterion/{name}/{name}_cached_5_32/base/estimates.json",
    ]
    result: list[BenchResult] = [BenchResult.from_file(p) for p in paths]
    result_cached: list[BenchResult] = [BenchResult.from_file(p) for p in paths_cached]

    baseline = result[0]

    result = [r.div(baseline) for r in result]
    result_cached = [r.div(baseline) for r in result_cached]

    return result, result_cached


ds = "sg_linear"
diamond_no_cache, diamond_cached = load_estimates(ds)

print(diamond_no_cache)
for r in diamond_no_cache:
    print(f"Mean: {r.mean.point_estimate} ns, Median: {r.median.point_estimate} ns, Std Dev: {r.std_dev.point_estimate} ns")

plb.init()
fig = plb.SuperFigure.subplots(nrows=1, ncols=1)
fig.set_default_plot_style()
x_ticks = ["1", "2", "5"]
y_data = [
    [r.mean.point_estimate * 100 for r in diamond_no_cache],
    [r.mean.point_estimate * 100 for r in diamond_cached]
]
X = np.arange(len(x_ticks))

fig.multiple_bars(y_data, width=0.35, bar_offset=0.01, labels=["No cache", "With cache"])

fig.set_text(
    title="Linear pattern (size = 32)",
    xlabel="Number of queries",
    ylabel="\% of baseline"
)
fig.set_xtick_label(x_ticks)


fig.show()