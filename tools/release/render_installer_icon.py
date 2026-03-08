from pathlib import Path

from PIL import Image, ImageDraw


ICON_SIZES = [(256, 256), (128, 128), (64, 64), (48, 48), (32, 32), (24, 24), (16, 16)]


def build_icon(size: int = 256) -> Image.Image:
    image = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(image)

    pad = int(size * 0.22)
    inner_pad = int(size * 0.42)
    ring_box = (pad, pad, size - pad, size - pad)
    inner_box = (inner_pad, inner_pad, size - inner_pad, size - inner_pad)
    ring_width = max(10, int(size * 0.11))

    dark = (22, 163, 74, 255)
    light = (52, 211, 153, 255)

    draw.ellipse(ring_box, outline=dark, width=ring_width)
    draw.arc(ring_box, start=210, end=25, fill=light, width=ring_width)
    draw.ellipse(inner_box, fill=(0, 0, 0, 0))
    return image


def main() -> None:
    repo_root = Path(__file__).resolve().parents[2]
    out_path = repo_root / "installer" / "windows" / "ocp-installer.ico"
    out_path.parent.mkdir(parents=True, exist_ok=True)

    icon = build_icon()
    icon.save(out_path, format="ICO", sizes=ICON_SIZES)
    print(out_path)


if __name__ == "__main__":
    main()
