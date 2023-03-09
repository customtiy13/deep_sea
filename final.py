import pickle
import time
from datetime import datetime
from numpy import array

DATE_FORMAT_STR = "%d/%m/%Y %H:%M:%S"
FILE_PATH = "clean_train_ais_records.pkl"
OUTPUT_FILE = "ct_custom.pkl"


LAT_MIN = 55.5
LAT_MAX = 58.0
LON_MIN = 10.3
LON_MAX = 13
SOG_MAX = 30
SOG_MIN = 0
COG_MAX = 72
COG_MIN = 0

RESULT = []


# The format of traj is [lat, log, sog, cog, unix_timestamp, mmsi]
#             value = [timestamp, lat, lon, cog, cog]


def run():
    with open(FILE_PATH, "rb") as f:
        data = pickle.load(f)
        for key, value in data.items():
            for voyage in value:
                if not voyage:
                    print("empty")
                    continue
                temp = {}
                traj = process_voyage(key, voyage)
                temp["mmsi"] = int(key)
                temp["traj"] = array(traj, dtype=float)
                RESULT.append(temp)
    export_to_pkl(RESULT)


def export_to_pkl(records):
    print("in exporting.")
    # print(DICT["538005505"])
    with open(OUTPUT_FILE, "wb") as f:
        pickle.dump(records, f)


def process_voyage(mmsi, voyage):
    traj = []
    for each in voyage:
        unix_timestamp = datetime.strptime(
            each[0], DATE_FORMAT_STR
        ).timestamp()
        # scale
        lat = (float(each[1]) - LAT_MIN) / (LAT_MAX - LAT_MIN)
        lon = (float(each[2]) - LON_MIN) / (LON_MAX - LON_MIN)
        sog = (float(each[3]) - SOG_MIN) / (SOG_MAX - SOG_MIN)
        cog = (float(each[4]) - COG_MIN) / (COG_MAX - COG_MIN)
        point = [lat, lon, sog, cog, unix_timestamp, mmsi]
        traj.append(point)

    return traj


if __name__ == "__main__":
    run()
