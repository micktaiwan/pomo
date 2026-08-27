#!/usr/bin/env python3
"""Draw the three dialog icons and build the .icns files pomo shows.

Run from the repo root: python3 tools/make-icons.py
Writes assets/icons/{once,repeat,alert}.icns. Pillow and iconutil (macOS) required.
"""
import math
import shutil
import subprocess
import sys
from pathlib import Path

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "assets" / "icons"
S = 1024  # master canvas; every icns size is downscaled from this
RADIUS = int(S * 0.22)
WHITE = (255, 255, 255, 255)


def plate(color):
    """Rounded-square background, the shape every macOS app icon wears."""
    img = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    ImageDraw.Draw(img).rounded_rectangle([0, 0, S - 1, S - 1], RADIUS, fill=color)
    return img


def draw_once():
    """A clock face: one reminder, at one time."""
    img = plate((44, 110, 214, 255))
    d = ImageDraw.Draw(img)
    cx = cy = S // 2
    r = int(S * 0.30)
    d.ellipse([cx - r, cy - r, cx + r, cy + r], outline=WHITE, width=int(S * 0.045))
    # Hands at 10:10, the pose every clock is photographed in.
    for angle_deg, length in ((-60, 0.20), (30, 0.15)):
        a = math.radians(angle_deg)
        d.line(
            [cx, cy, cx + math.cos(a) * S * length, cy + math.sin(a) * S * length],
            fill=WHITE,
            width=int(S * 0.040),
        )
    d.ellipse([cx - S // 40, cy - S // 40, cx + S // 40, cy + S // 40], fill=WHITE)
    return img


def draw_repeat():
    """A loop closed by an arrowhead: the reminder comes back."""
    img = plate((22, 148, 116, 255))
    d = ImageDraw.Draw(img)
    cx = cy = S // 2
    r = int(S * 0.28)
    w = int(S * 0.055)
    # Open arc, the gap left for the arrowhead to sit in.
    d.arc([cx - r, cy - r, cx + r, cy + r], start=310, end=210, fill=WHITE, width=w)
    tip = int(S * 0.085)
    ax, ay = cx + r, cy  # gap centre, at 3 o'clock
    d.polygon(
        [(ax + tip * 0.9, ay + tip * 0.2), (ax - tip * 0.9, ay + tip * 0.2), (ax, ay - tip * 1.1)],
        fill=WHITE,
    )
    return img


def draw_alert():
    """A warning triangle: something needs an eye on it now."""
    img = plate((199, 48, 42, 255))
    d = ImageDraw.Draw(img)
    cx = S // 2
    top, bottom, half = int(S * 0.20), int(S * 0.78), int(S * 0.33)
    d.polygon([(cx, top), (cx - half, bottom), (cx + half, bottom)], fill=WHITE)
    bar_w = int(S * 0.055)
    d.rounded_rectangle(
        [cx - bar_w, int(S * 0.38), cx + bar_w, int(S * 0.60)],
        bar_w,
        fill=(199, 48, 42, 255),
    )
    dot = int(S * 0.058)
    d.ellipse(
        [cx - dot, int(S * 0.645) - dot, cx + dot, int(S * 0.645) + dot],
        fill=(199, 48, 42, 255),
    )
    return img


ICONS = {"once": draw_once, "repeat": draw_repeat, "alert": draw_alert}
# iconutil expects exactly these names inside the .iconset directory.
SIZES = [(16, 1), (16, 2), (32, 1), (32, 2), (128, 1), (128, 2), (256, 1), (256, 2), (512, 1), (512, 2)]


def build(name, master):
    iconset = OUT / f"{name}.iconset"
    if iconset.exists():
        shutil.rmtree(iconset)
    iconset.mkdir(parents=True)
    for size, scale in SIZES:
        px = size * scale
        suffix = "" if scale == 1 else "@2x"
        master.resize((px, px), Image.LANCZOS).save(iconset / f"icon_{size}x{size}{suffix}.png")
    subprocess.run(
        ["iconutil", "-c", "icns", str(iconset), "-o", str(OUT / f"{name}.icns")], check=True
    )
    shutil.rmtree(iconset)


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    for name, draw in ICONS.items():
        build(name, draw())
        print(f"wrote {OUT / f'{name}.icns'}")


if __name__ == "__main__":
    sys.exit(main())
