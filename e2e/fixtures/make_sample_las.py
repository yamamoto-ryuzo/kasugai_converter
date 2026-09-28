"""Generate a small LAS point cloud for E2E tests.

Usage: <python> make_sample_las.py <output.las>
Requires laspy (the project's tools/python distribution has it).
"""

import sys

import numpy as np
import laspy


def main() -> None:
    out = sys.argv[1]

    header = laspy.LasHeader(point_format=0, version="1.2")
    header.scales = np.array([0.001, 0.001, 0.001])
    header.offsets = np.array([0.0, 0.0, 0.0])

    las = laspy.LasData(header)

    axis = np.linspace(0.0, 50.0, 20)
    heights = np.linspace(0.0, 10.0, 5)
    xs, ys, zs = np.meshgrid(axis, axis, heights)
    las.x = xs.ravel()
    las.y = ys.ravel()
    las.z = zs.ravel()

    las.write(out)
    print(f"wrote {out} ({len(las.x)} points)")


if __name__ == "__main__":
    main()
