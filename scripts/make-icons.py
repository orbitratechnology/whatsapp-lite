#!/usr/bin/env python3
"""Generate WhatsApp Lite's simple chat-bubble app icon in Tauri bundle formats."""
from io import BytesIO
from pathlib import Path

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[1]
ICON_DIR = ROOT / "src-tauri" / "icons"
ICON_DIR.mkdir(parents=True, exist_ok=True)
SCALE = 3
SIZE = 1024

# Draw at 3x then downsample for clean edges.
canvas = Image.new("RGBA", (SIZE * SCALE, SIZE * SCALE), (0, 0, 0, 0))
draw = ImageDraw.Draw(canvas)

def s(value: int) -> int:
    return value * SCALE

# Rounded app tile.
draw.rounded_rectangle((s(36), s(36), s(988), s(988)), radius=s(230), fill="#08765E")
# White message bubble and lower-left tail.
draw.ellipse((s(168), s(155), s(856), s(843)), fill="#FFFFFF")
draw.polygon([(s(238), s(682)), (s(207), s(822)), (s(368), s(774)), (s(390), s(675))], fill="#FFFFFF")
# A clean, abstract handset stroke.
points = [(390, 350), (364, 397), (370, 445), (397, 496), (438, 541), (483, 580), (531, 606), (580, 617), (623, 601), (653, 570)]
scaled = [(s(x), s(y)) for x, y in points]
draw.line(scaled, fill="#08765E", width=s(76), joint="curve")
# End-cap strokes evoke a handset without fine detail that disappears at tray sizes.
draw.line([(s(389), s(350)), (s(433), s(389))], fill="#08765E", width=s(76))
draw.line([(s(614), s(563)), (s(653), s(570))], fill="#08765E", width=s(76))
for x, y in (points[0], points[-1]):
    draw.ellipse((s(x - 38), s(y - 38), s(x + 38), s(y + 38)), fill="#08765E")

image = canvas.resize((SIZE, SIZE), Image.Resampling.LANCZOS)
image.save(ICON_DIR / "icon.png", optimize=True)
image.save(
    ICON_DIR / "icon.ico",
    format="ICO",
    sizes=[(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)],
)

# ICNS is a small container of PNG payloads tagged with their Apple size codes.
chunks = []
for size, code in [(16, b"icp4"), (32, b"icp5"), (64, b"icp6"), (128, b"ic07"), (256, b"ic08"), (512, b"ic09"), (1024, b"ic10")]:
    stream = BytesIO()
    image.resize((size, size), Image.Resampling.LANCZOS).save(stream, format="PNG", optimize=True)
    payload = stream.getvalue()
    chunks.append(code + (len(payload) + 8).to_bytes(4, "big") + payload)
body = b"".join(chunks)
(ICON_DIR / "icon.icns").write_bytes(b"icns" + (len(body) + 8).to_bytes(4, "big") + body)
print(f"Wrote bundle icons to {ICON_DIR}")
