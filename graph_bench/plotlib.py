from enum import Enum

import matplotlib
import matplotlib.pyplot as plt
import numpy as np


class FigSize(Enum):
    SMALL = (8, 6)
    MEDIUM = (12, 9)
    BIG = (16, 12)
    OVERLEAF = (10, 6)  # Adjust this value as needed
    MICRO = (3, 2)

class Color(Enum):
    GREEN = tuple(x / 255 for x in (58, 153, 95))
    PURPLE = tuple(x / 255 for x in (144, 103, 167))
    RED = tuple(x / 255 for x in (211, 95, 96))
    BLUE = tuple(x / 255 for x in (39, 84, 138))
    ORANGE = tuple(x / 255 for x in (201, 149, 71))
    YELLOW = tuple(x / 255 for x in (199, 194, 103))
    PINK = tuple(x / 255 for x in (140, 79, 104))
    CYAN = tuple(x / 255 for x in (70, 122, 120))
    TUDELFT = tuple(x / 255 for x in (110, 187, 213))

    def rgb(self, luminance=0):
        lumi = luminance / 255
        return tuple(val + lumi for val in self.value)

    def hsl(self, luminance=0):
        rgb = self.rgb(luminance)
        return matplotlib.colors.rgb_to_hsv(rgb)

    def __str__(self):
        return self.name.lower()

class FontSize(Enum):
    SMALL = 12
    MEDIUM = 16
    LARGE = 18

    @classmethod
    def label(cls):
        return cls.MEDIUM.value

    @classmethod
    def legend(cls):
        return cls.MEDIUM.value

    @classmethod
    def title(cls):
        return cls.LARGE.value

    @classmethod
    def ticks(cls):
        return cls.SMALL.value

# FONT_SMALL = 12
# FONT_MEDIUM = 16
# FONT_LARGE = 18

LINE_STYLE = "--"

# I'm calling it superfigure and no one can stop me
class SuperFigure:
    __fig: matplotlib.figure.Figure
    __ax: matplotlib.axes.Axes

    __is_init = False

    def __init__(self, _fig: matplotlib.figure.Figure, _ax: matplotlib.axes.Axes) -> None:
        self.__fig = _fig
        self.__ax = _ax

    def init_plb():
        """Initialise some plot constants (call this first):
        - Sets color order to my custom colors
        - Enables latex font

        Args:
            plt (_type_): _description_
        """

        if SuperFigure.__is_init:
            return

        print("Initialising plot constants...")
        SuperFigure.__set_color_order()
        SuperFigure.__set_latex_font()
        SuperFigure.__is_init = True

    # def fig(self):
    #     """Get the figure of this SuperFigure"""
    #     return self.__fig

    # def ax(self):
    #     """Get the axes of this SuperFigure"""
    #     return self.__ax

    def figure(**kwargs) -> 'SuperFigure':
        SuperFigure.init_plb()
        fix, axs = plt.figure(**kwargs)
        return SuperFigure(fix, axs)

    def subplots(nrows=1, ncols=1, **kwargs):
        """Create subplots and return a SuperFigure object"""
        SuperFigure.init_plb()
        fig, axs = plt.subplots(nrows=nrows, ncols=ncols, **kwargs)
        return SuperFigure(fig, axs)


    def set_default_plot_style(self):
        """Set plot style to default style that I prefer:
        - Enables grid
        - Sets figure size to be relatively large, good for reports
        - Enables major and minor ticks on x and y axes

        Args:
            fig (_type_): Matplotlib figure
            ax (_type_): matplotlib axes
        """
        self.set_size(FigSize.OVERLEAF)

        SuperFigure.__set_default_ax_style(self.__ax)


    def set_default_subplot_style(self):
        """Set plot style to default style that I prefer:
        - Enables grid
        - Sets figure size to be relatively large, good for reports
        - Enables major and minor ticks on x and y axes

        IMPORTANT: this is used when you have multiple subplots

        Args:
            fig (_type_): Matplotlib figure
            ax (_type_): matplotlib axes
        """
        self.set_size(FigSize.OVERLEAF)
        self.__fig.tight_layout(pad=1.0)
        rows, cols = self.__axs.shape
        for r in range(rows):
            for c in range(cols):
                SuperFigure.__set_default_ax_style(self.__ax[r, c])

    def set_size(self, size: FigSize):
        """Set figure size

        Args:
            fig (Matplotlib Figure): Figure to change size of
            size (str): size in string format, allowed values: `{'s', 'm', 'b', 'o', 'u', 'f'}` for small, medium, big, overleaf/report size, micro size and fullscreen respectively.
        """
        # if not isinstance(size, (tuple)):
        #     size = __str2size(size)

        self.__fig.set_size_inches(size.value)

    def save_figure(self, file_name, file_extension='png'):
        """
        Save a Matplotlib figure to a file in the specified format (PNG, JPG, or EPS).

        Parameters:
            - fig: Matplotlib figure object.
            - file_name: Name of the output file (excluding the extension).
            - file_extension: File extension for saving the figure ('png', 'jpg', or 'eps').

        Returns:
            None
        """
        supported_extensions = ['png', 'jpg', 'jpeg', 'eps', 'svg']

        # Check if the specified file extension is supported
        if file_extension.lower() not in supported_extensions:
            raise ValueError("Unsupported file extension. Supported extensions are: png, jpg, jpeg, eps.")

        # Determine the format and quality for JPG images
        if file_extension.lower() in ['jpg', 'jpeg']:
            format_str = 'jpg'
        else:
            format_str = file_extension.lower()

        # Save the figure with the specified file extension
        file_path = f"{file_name}.{format_str}"

        self.__fig.savefig(file_path, format=format_str, dpi=200, bbox_inches='tight')

        print(f"Figure saved as '{file_path}'.")

    def __set_default_ax_style(ax):
        ax.grid(True, linestyle=LINE_STYLE, alpha=0.5)
        ax.tick_params(axis='both', which='major', labelsize=FontSize.ticks(), width=1.2, length=6)
        ax.tick_params(axis='both', which='minor', width=0.8, length=3)

    def set_origin_ax_line(self):
        self.__ax.axhline(y=0, color='black', alpha=0.5, linestyle=LINE_STYLE)
        self.__ax.axvline(x=0, color='black', alpha=0.5, linestyle=LINE_STYLE)

    def set_text(self, title: str, xlabel: str, ylabel: str, legend_loc='best'):
        """Set title and x/y labels for ax. Also enables legend

        Args:
            ax (Matplotlib Axes): Axes of plot that should be changed (use plt.subplots() to get one)
            title (str): Title of plot
            xlabel (str): X label of plot
            ylabel (str): Y label of plot
        """
        self.__ax.set_title(title, fontsize=FontSize.title())
        self.__ax.set_xlabel(xlabel, fontsize=FontSize.label())
        self.__ax.set_ylabel(ylabel, fontsize=FontSize.label())
        self.__ax.legend(fontsize=FontSize.legend(), loc=legend_loc)

    def set_xtick_label(self, labels):
        """Set x tick labels for ax

        Args:
            ax (Matplotlib Axes): Axes of plot that should be changed (use plt.subplots() to get one)
            labels (list): List of labels
        """
        self.__ax.set_xticks(np.arange(len(labels)))
        self.__ax.set_xticklabels(labels)

    def set_xtick_angle(self, angle=0.0):
        plt.setp(self.__ax.get_xticklabels(), rotation=angle, ha="right", rotation_mode="anchor")

    def set_y_log(self):
        """Set y axis to log scale

        Args:
            ax (Matplotlib Axes): Axes of plot that should be changed (use plt.subplots() to get one)
        """
        self.__ax.set_yscale('log')

    def set_lim(self, xlim=None, ylim=None):
        """Set x and y limits for ax

        Args:
            ax (Matplotlib Axes): Axes of plot that should be changed (use plt.subplots() to get one)
            xlim (tuple): x limits
            ylim (tuple): y limits
        """
        if xlim is not None:
            self.__ax.set_xlim(xlim)

        if ylim is not None:
            self.__ax.set_ylim(ylim)

    def plot(self, *args, **kwargs):
        """Plot data on the ax

        Args:
            *args: Positional arguments for plt.plot()
            **kwargs: Keyword arguments for plt.plot()
        """
        self.__ax.plot(*args, **kwargs)

    def bar(self, *args, **kwargs):
        self.__ax.bar(*args, **kwargs)

    def boxplot(self, *args, **kwargs):
        """Create a boxplot on the ax

        Args:
            *args: Positional arguments for plt.boxplot()
            **kwargs: Keyword arguments for plt.boxplot()
        """
        self.__ax.boxplot(*args, **kwargs)

    def show(self):
        """Show the plot"""
        plt.show()

    def multiple_bars(self, y_data, labels: list[str], colors: list[Color], width=0.1, bar_offset=0, **kwargs):
        num_bars = len(y_data)
        num_labels = len(y_data[0])
        X = np.arange(num_labels)
        for i, y in enumerate(y_data):
            i_o = i - num_bars / 2
            offset_x = X + (width * i_o) + (bar_offset * i_o)
            offset_x += width / 2 + bar_offset / 2

            self.__ax.bar(offset_x, y, width=width, label=labels[i], color=colors[i], **kwargs)

    def get_color_order(luminance=25):
        return [
            Color.GREEN.rgb(luminance),
            Color.PURPLE.rgb(luminance),
            Color.RED.rgb(luminance),
            Color.BLUE.rgb(luminance),
            Color.ORANGE.rgb(luminance),
            Color.YELLOW.rgb(luminance),
            Color.PINK.rgb(luminance),
            Color.CYAN.rgb(luminance),
        ]

    def __str2size(size_str):
        # Get screen resolution in pixels
        screen_resolution = plt.gcf().dpi

        if size_str in ['small', 's']:
            size = (8, 6)
        elif size_str in ['medium', 'm']:
            size = (12, 9)
        elif size_str in ['big', 'b']:
            size = (16, 12)
        elif size_str in ['overleaf', 'report', 'r', 'o']:
            size = (10, 6)  # Adjust this value as needed
        elif size_str in ['fullscreen', 'full', 'f']:
            size = (plt.gcf().get_window_extent().width / screen_resolution, plt.gcf().get_window_extent().height / screen_resolution)
        elif size_str in ['micro', 'u']:
            size = (3, 2)
        else:
            size = (8, 6)

        # Convert size to pixels
        # size = [val * screen_resolution for val in size]

        return size


    def __set_color_order():
        colors = SuperFigure.get_color_order()
        plt.rcParams['axes.prop_cycle'] = plt.cycler(color=colors)

    def __set_latex_font():
        plt.rcParams.update({
            "text.usetex": True,
            "font.family": "serif",  # You can change this to "sans-serif" if you prefer
            "font.size": 12,         # Adjust the font size as needed
        })