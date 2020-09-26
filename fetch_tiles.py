from math import floor
import urllib.request

def conv(lat, long):
    zoom = 3
    x = (long + 180.0) / 360.0 * 2.0 * (3.0 ** zoom)
    y = (90.0 - lat) / 180.0 * (3.0**zoom)

    return (x, y)

top_left = conv(65.09, -13.3)
bottom_right = conv(12.36, 63.14)

for x in range(floor(top_left[0]), floor(bottom_right[0]) + 1):
    for y in range(floor(top_left[1]), floor(bottom_right[1]) + 1):
        print(f"{x}, {y}")

        with urllib.request.urlopen(f"https://terrariumearth.azureedge.net/geo3/elevation2/3/{x}/{y}") as r:
            with open(f"assets/heightmap/{x - 25}x{y - 3}.heightmap", "wb") as f:
                f.write(r.read())
