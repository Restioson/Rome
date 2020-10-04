from math import floor
import urllib.request
import os
import os.path
import zipfile

def conv(lat, long):
    zoom = 3
    x = (long + 180.0) / 360.0 * 2.0 * (3.0 ** zoom)
    y = (90.0 - lat) / 180.0 * (3.0**zoom)

    return (floor(x), floor(y))

os.makedirs("data/heightmap", exist_ok=True)
os.makedirs("data/water_polygons", exist_ok=True)

print("Downloading heightmap tiles")
top_left = conv(65.09, -24.3)
bottom_right = conv(12.36, 63.14)

for x in range(top_left[0], bottom_right[0] + 1):
    for y in range(top_left[1], bottom_right[1] + 1):
        print(f"Downloading tile at {x}, {y}")

        path = f"data/heightmap/{x - top_left[0]}x{y - top_left[1]}.heightmap"
        if os.path.exists(path):
            print(" -> Skipping (file exists)")
            continue

        with urllib.request.urlopen(f"https://terrariumearth.azureedge.net/geo3/elevation2/3/{x}/{y}") as r:
            with open(path, "wb") as f:
                f.write(r.read())

print("===============================")
print("Downloading OSM water polygons")

if not os.path.exists("data/water_polygons/water_polygons.shp"):
    zip_path = "data/water_polygons/polygons.zip"
    with urllib.request.urlopen(f"https://osmdata.openstreetmap.de/download/water-polygons-split-4326.zip") as r:
        with open(zip_path, "wb") as f:
            f.write(r.read())
    print("Unzipping")
    with zipfile.ZipFile(zip_path, "r") as zip:
        zip.extract("water-polygons-split-4326/water_polygons.shp", "data/water_polygons/water_polygons.shp")

    os.remove(zip_path)
else:
    print(" -> Skipping (file exists)")

print("===============================")
print("Done downloading data.")
