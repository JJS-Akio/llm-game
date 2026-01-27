from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Iterable, Mapping

from PIL import Image

ASSETS_DIR = Path(__file__).resolve().parent
BASE_SIZE = 16
SCALE = 2
SIZE = BASE_SIZE * SCALE

# Colors (RGBA)
TRANSPARENT = (0, 0, 0, 0)
BLACK = (0, 0, 0, 255)
GREY_FULL_DARK = (140, 140, 140, 255)
GREY_HALF_FILLED = (185, 185, 185, 255)
GREY_HALF_EMPTY = (115, 115, 115, 255)
GREY_EMPTY_LIGHT = (210, 210, 210, 255)


@dataclass(frozen=True)
class IconShape:
    outline: set[tuple[int, int]]
    fill: set[tuple[int, int]]


def points(rows: Iterable[str]) -> set[tuple[int, int]]:
    pts: set[tuple[int, int]] = set()
    for y, row in enumerate(rows):
        for x, ch in enumerate(row):
            if ch != ".":
                pts.add((x, y))
    return pts


def boundary(points_set: set[tuple[int, int]]) -> set[tuple[int, int]]:
    if not points_set:
        return set()
    neighbors = [
        (-1, -1),
        (0, -1),
        (1, -1),
        (-1, 0),
        (1, 0),
        (-1, 1),
        (0, 1),
        (1, 1),
    ]
    edge: set[tuple[int, int]] = set()
    for x, y in points_set:
        if any((x + dx, y + dy) not in points_set for dx, dy in neighbors):
            edge.add((x, y))
    return edge


def draw_icon(name: str, shape: IconShape, half_mode: str = "left") -> None:
    def render_variant(
        variant: str,
        outline_color: tuple[int, int, int, int],
        filled: set[tuple[int, int]],
        empty: set[tuple[int, int]],
        filled_color: tuple[int, int, int, int],
        empty_color: tuple[int, int, int, int],
    ):
        img = Image.new("RGBA", (SIZE, SIZE), TRANSPARENT)
        px = img.load()

        def plot_scaled(x: int, y: int, color: tuple[int, int, int, int]) -> None:
            sx = x * SCALE
            sy = y * SCALE
            for dx in range(SCALE):
                for dy in range(SCALE):
                    px[sx + dx, sy + dy] = color

        for x, y in empty:
            plot_scaled(x, y, empty_color)
        for x, y in filled:
            plot_scaled(x, y, filled_color)
        outline_points = shape.outline or boundary(shape.fill)
        for x, y in outline_points:
            plot_scaled(x, y, outline_color)
        out_path = ASSETS_DIR / f"{name}_{variant}.png"
        img.save(out_path)

    fill_points = set(shape.fill)
    if not fill_points:
        return

    max_x = max(x for x, _ in fill_points) + 1
    max_y = max(y for _, y in fill_points) + 1
    half_x = max_x / 2.0
    half_y = max_y / 2.0

    if half_mode == "top":
        # For stamina we want the bottom greyed out first.
        half_points = {p for p in fill_points if p[1] < half_y}
    else:
        # Default is left-to-right fill for health and food.
        half_points = {p for p in fill_points if p[0] < half_x}
    empty_points = fill_points - half_points

    render_variant(
        "full",
        BLACK,
        fill_points,
        set(),
        GREY_FULL_DARK,
        GREY_FULL_DARK,
    )
    render_variant(
        "half",
        BLACK,
        half_points,
        empty_points,
        GREY_HALF_FILLED,
        GREY_HALF_EMPTY,
    )
    # Empty: black outline with a lighter grey interior.
    render_variant(
        "empty",
        BLACK,
        set(),
        fill_points,
        GREY_EMPTY_LIGHT,
        GREY_EMPTY_LIGHT,
    )


def build_heart() -> IconShape:
    outline_rows = [
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
    ]
    fill_rows = [
        "................",
        "................",
        "................",
        "...XXXX..XXXX...",
        "..XXXXXXXXXXXX..",
        ".XXXXXXXXXXXXXX.",
        ".XXXXXXXXXXXXXX.",
        "..XXXXXXXXXXXX..",
        "...XXXXXXXXXX...",
        "....XXXXXXXX....",
        ".....XXXXXX.....",
        "......XXXX......",
        ".......XX.......",
        "................",
        "................",
        "................",
    ]
    return IconShape(points(outline_rows), points(fill_rows))


def build_steak() -> IconShape:
    outline_rows = [
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
    ]
    fill_rows = [
        "................",
        "................",
        "....XXXXXX......",
        "..XXXXXXXXXX....",
        ".XXXXXXXXXXXX...",
        "XXXXXXXX...XXX..",
        "XXXXXXX..XX..XX.",
        "XXXXXXX..XX..XXX",
        "XXXXXXXX...XXXXX",
        ".XXXXXXXXXXXXXXX",
        "..XXXXXXXXXXXXX.",
        "...XXXXXXXXXXX..",
        "...XXXXXXXXX....",
        "....XXXXX.......",
        "................",
        "................",
    ]
    return IconShape(points(outline_rows), points(fill_rows))


def build_lightning() -> IconShape:
    outline_rows = [
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
    ]
    fill_rows = [
        "..XXXXXXXXXXXX..",
        ".XXXXXXXXXXXXXX.",
        ".XXXXXXXXXXXX...",
        "..XXXXXXXXXX....",
        "...XXXXXXXX.....",
        "...XXXXXXX......",
        "..XXXXXXXXXX....",
        "....XXXXXXXXX...",
        ".....XXXXXXXX...",
        "....XXXXXXX.....",
        "...XXXXXXX......",
        "...XXXXXX.......",
        "..XXXXXX........",
        "..XXXXX.........",
        ".XXXX...........",
        "................",
    ]
    return IconShape(points(outline_rows), points(fill_rows))


def main() -> None:
    shapes: Mapping[str, IconShape] = {
        "health": build_heart(),
        "food": build_steak(),
        "stamina": build_lightning(),
    }
    for name, shape in shapes.items():
        half_mode = "top" if name == "stamina" else "left"
        draw_icon(name, shape, half_mode=half_mode)
    print("Generated status icons in", ASSETS_DIR)


if __name__ == "__main__":
    main()
