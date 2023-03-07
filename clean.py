import os
import csv
import pickle
import math
import torch
import logging
import numpy as np

import pandas as pd
from datetime import datetime
from haversine import haversine
from haversine import haversine_vector

import matplotlib.pyplot as plt


DEVICE=torch.device("cpu")
DATE_FORMAT_STR = "%d/%m/%Y %H:%M:%S"
DATA_DIR = "/home/mys/master/papers/sea/data/deep_sea/clean_test"
DICT = {}  # save all the results
RESULT = {}
LIMIT = 1.852
# D = np.load("./coastal_basemap_data.npy", allow_pickle=True).tolist()
F = torch.from_numpy(np.load("./author_coastal_basemap_data.npy", allow_pickle=True)).to(DEVICE)



def save_coastal_data2(path):
    with open("dma_coastline_polygons.pkl", "rb") as f:
        a = pickle.load(f)
        a = np.concatenate(a, axis=0)
        func = np.vectorize(math.radians)
        a = func(a)
        np.save(os.path.join(path, "author_coastal_basemap_data.npy"), a)


# def save_coastal_data(path, resolution="f"):
# m = Basemap(
# projection="merc",
# llcrnrlon=10,
# llcrnrlat=55.5,
# urcrnrlon=13.0,
# urcrnrlat=58.0,
# resolution=resolution,
# )

# coast = m.drawcoastlines()

# coordinates = np.vstack(coast.get_segments())
# lons, lats = m(coordinates[:, 0], coordinates[:, 1], inverse=True)

# D = {"lons": lons, "lats": lats}
# np.save(os.path.join(path, "coastal_basemap_data.npy"), D)


# def distance_from_coast(lon, lat, degree_in_km=111.12):
# lons, lats = D["lons"], D["lats"]

# dists = np.sqrt((lons - lon) ** 2 + (lats - lat) ** 2)

# return np.min(dists) * degree_in_km


def distance_from_coast2(all, seq_len):
    # a = np.zeros(F.shape) + [math.radians(lat), math.radians(lon)]
    # a = torch.from_numpy(np.array([math.radians(lat), math.radians(lon)]).reshape(1, -1)).to(DEVICE)
    all = np.zeros((seq_len, F.shape[0], 2)) + all.reshape(seq_len, -1, 2)
    a = torch.from_numpy(all).to(DEVICE)
    lat1 = a[:,:, 0]
    lon1 = a[:,:, 1]
    lat2 = F[:, 0]
    lon2 = F[:, 1]

    diff_lat = lat1 - lat2
    diff_lon = lon1 - lon2
    d = (
        torch.sin(diff_lat / 2) ** 2
        + torch.cos(lat1) * torch.cos(lat2) * torch.sin(diff_lon / 2) ** 2
    )

    b = torch.min(2 * 6371 * torch.arcsin(torch.sqrt(d)), -1).values
    return b


def helper(row):
    lat = row[0]
    lon = row[1]
    t_lat = row[2]
    t_lon = row[3]

    return haversine((lat, lon), (t_lat, t_lon))


def run():
    i = 0
    for root, _, files in os.walk(DATA_DIR):
        for file in sorted(files):
            file_path = os.path.join(root, file)
            process_file(file_path)
            # print(file_path)
            # break  # TODO, delete me
    # process_file("/home/mys/master/papers/sea/data/deep_sea/vanilla/a")

    # delete useless items
    clean_dict()
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
    all = []
    data = []
    mmsis = []
    count = 1
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
            value = [timestamp, lat, lon, sog, cog]
            point = [math.radians(lat), math.radians(lon)]
            all.append(point)
            data.append(value)
            mmsis.append(mmsi)
            if len(all) == 128:
                count += 1
                mm = distance_from_coast2(np.asarray(all), len(all))
                mm = (mm < LIMIT).tolist()
                for i, v in enumerate(mm):
                    if mm[i]:
                        continue
                    DICT.setdefault(mmsis[i], []).append(data[i])
                all = []
                data = []
                mmsis = []
                print(f"current count is {count}")
    if all:
        mm = distance_from_coast2(np.asarray(all), len(all))
        mm = (mm < LIMIT).tolist()
        for i, v in enumerate(mm):
            if mm[i]:
                continue
            DICT.setdefault(mmsis[i], []).append(data[i])
        all = []
        data = []
        mmsis = []
    
    # with open(file_path) as f:
        # rdr = csv.reader(f, delimiter=",")
        # next(rdr)
        # for i, row in enumerate(rdr):
            # if mm[i]:
                # continue
            # timestamp, mmsi, lat, lon, sog, cog = (
                # row[0],
                # row[1],
                # float(row[2]),
                # float(row[3]),
                # row[4],
                # row[5],
            # )
            # DICT.setdefault(mmsi, []).append(value)
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
        ret = split_to_short(ret)

        if ret and ret[0]:
            # the end
            RESULT[key] = ret


def split_to_short(values):
    result = []
    for voyage in values:
        split_idx = 0
        last = voyage[0]
        for i in range(1, len(voyage)):
            diff = compare_time_diff(last[0], voyage[i][0])
            if diff > 20:
                # split
                result.append(voyage[split_idx:i])
                split_idx = i
                last = voyage[i]
        result.append(voyage[split_idx:i])
    return result


def down_sample(values):
    result = []
    for voyage in values:
        start = voyage[0]
        ret = [start]
        for i in range(1, len(voyage)):
            diff = compare_time_diff(start[0], voyage[i][0])
            if diff >= 10 / 60:  # TODO. 10 miniutes
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


OUTPUT_FILE = "clean_test_ais_records.pkl"


def export_to_pkl(records):
    print("in exporting.")
    # print(DICT["538005505"])
    with open(OUTPUT_FILE, "wb") as f:
        pickle.dump(records, f)


if __name__ == "__main__":
    # run()
    # save_coastal_data2("./")
    run()
