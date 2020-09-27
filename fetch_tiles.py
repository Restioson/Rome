from math import floor
import urllib.request

def conv(lat, long):
    zoom = 3
    x = (long + 180.0) / 360.0 * 2.0 * (3.0 ** zoom)
    y = (90.0 - lat) / 180.0 * (3.0**zoom)

    return (floor(x), floor(y))

top_left = conv(65.09, -24.3)
bottom_right = conv(12.36, 63.14)

for x in range(top_left[0], bottom_right[0] + 1):
    for y in range(top_left[1], bottom_right[1] + 1):
        print(f"{x}, {y}")

        with urllib.request.urlopen(f"https://terrariumearth.azureedge.net/geo3/elevation2/3/{x}/{y}") as r:
            with open(f"assets/heightmap/{x - top_left[0]}x{y - top_left[1]}.heightmap", "wb") as f:
                f.write(r.read())
