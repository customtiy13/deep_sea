import os
import csv
import pickle
import logging
import numpy as np
import pandas as pd
from datetime import datetime
from haversine import haversine

import matplotlib.pyplot as plt

from mpl_toolkits.basemap import Basemap

DATE_FORMAT_STR = "%d/%m/%Y %H:%M:%S"
DATA_DIR = "/home/mys/master/papers/sea/data/deep_sea/vanilla"
DICT = {}  # save all the results
RESULT = {}
LIMIT = 1.852
D = np.load("./coastal_basemap_data.npy", allow_pickle=True).tolist()


def save_coastal_data(path, resolution="f"):
    m = Basemap(
        projection="merc",
        llcrnrlon=10,
        llcrnrlat=55.5,
        urcrnrlon=13.0,
        urcrnrlat=58.0,
        resolution=resolution,
    )

    coast = m.drawcoastlines()

    coordinates = np.vstack(coast.get_segments())
    lons, lats = m(coordinates[:, 0], coordinates[:, 1], inverse=True)

    D = {"lons": lons, "lats": lats}
    np.save(os.path.join(path, "coastal_basemap_data.npy"), D)


def distance_from_coast(lon, lat, degree_in_km=111.12):
    lons, lats = D["lons"], D["lats"]

    dists = np.sqrt((lons - lon) ** 2 + (lats - lat) ** 2)

    return np.min(dists) * degree_in_km


def run():
    i = 0
    for root, _, files in os.walk(DATA_DIR):
        for file in sorted(files):
            i += 1
            if i > 1:
                break
            file_path = os.path.join(root, file)
            process_file(file_path)
            # print(file_path)
            # break  # TODO, delete me
    # process_file("/home/mys/master/papers/sea/data/deep_sea/vanilla/a")

    # delete useless items
    clean_dict()
    print(RESULT["538005505"])
    print("-------------------")
    export_to_pkl(RESULT)


def process_file(file_path):
    """
    pub struct ZRecord {
        pub timestamp: String,
        pub mmsi: String,
        pub lat: f64,
        pub lon: f64,
        pub sog: f64,
        pub cog: f64,
        pub ship_type: ShipType,
    }
    """
    print(f"in processing file {file_path}")
    with open(file_path) as f:
        rdr = csv.reader(f, delimiter=",")
        next(rdr)
        for row in rdr:
            timestamp, mmsi, lat, lon, sog, cog = (
                row[0],
                row[1],
                float(row[2]),
                float(row[3]),
                row[4],
                row[5],
            )
            if distance_from_coast(lon, lat) < LIMIT:
                continue
            # partial filter
            value = [timestamp, lat, lon, sog, cog]
            DICT.setdefault(mmsi, []).append(value)
    # print(DICT["538005505"])


def clean_dict():
    if not DICT:
        logging.error("DICT is NULL")

    print("in cleaning dict")

    # {mmsi: [[time,lat,lon, ~~], [time, lat, lon, ~~]]}
    # print(f"kkk {DICT['538005706']}")
    for key, value in DICT.items():
        ret = []
        # 1. TODO split
        last = value[0]
        split_idx = 0
        for i in range(1, len(value)):
            v = value[i]
            diff_in_hours = compare_time_diff(last[0], v[0])

            if diff_in_hours > 2:
                # split
                ret.append(value[split_idx:i])
                split_idx = i
            last = v
        ret.append(value[split_idx:i])
        # [[[mmsi:~], [mmsi:~]], [[], []]]
        # print(ret)


        # 2. filter
        ret = filter_voyages(ret)

        # 3. remove abnormal messages.
        ret = filter_abnormal(ret)

        # 4. downsample 10 miniu
        ret = down_sample(ret)

        # 5. split long into short

        # the end
        RESULT[key] = ret


def down_sample(values):
    result = []
    for voyage in values:
        start = voyage[0]
        ret = [start]
        for i in range(1, len(voyage)):
            diff = compare_time_diff(start[0], voyage[i][0])
            if diff >= 0.1:
                ret.append(voyage[i])
                start = voyage[i]
        result.append(ret)
    return result


def filter_abnormal(values):
    result = []
    for voyage in values:
        last = voyage[0]
        ret = [last]
        for i in range(1, len(voyage)):
            speed = caculate_speed(last, voyage[i])
            # 1kn = 1.852 km/h
            if speed <= 40 * 1.852 and speed != 0:
                ret.append(voyage[i])
            last = voyage[i]
        result.append(ret)
    return result


def caculate_speed(a, b):
    time = compare_time_diff(a[0], b[0])
    if time == 0:
        # print(a, b)
        return 0
    a_point = (float(a[1]), float(a[2]))
    b_point = (float(b[1]), float(b[2]))
    # (lat, lon)
    distance = haversine(a_point, b_point)  # in km
    speed = distance / time
    return speed


def filter_voyages(values):
    result = []
    ret = list(filter(lambda x: len(x) >= 20, values))
    for value in ret:
        diff = compare_time_diff(value[0][0], value[-1][0])
        if diff >= 4:
            result.append(value)
    return result


def compare_time_diff(atime, btime):
    start = datetime.strptime(atime, DATE_FORMAT_STR)
    end = datetime.strptime(btime, DATE_FORMAT_STR)
    diff_in_hours = (end - start).total_seconds() / 3600

    return diff_in_hours


OUTPUT_FILE = "ais_records.pkl"


def export_to_pkl(records):
    print("in exporting.")
    # print(DICT["538005505"])
    with open(OUTPUT_FILE, "wb") as f:
        pickle.dump(records, f)


if __name__ == "__main__":
    # run()
    # save_coastal_data("./")
    run()