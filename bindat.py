import json
import numpy as np
from dataclasses import dataclass
from typing import Self, Any

@dataclass
class BinDat:
    metadata: Any
    datasets: list[np.ndarray]

    @classmethod
    def from_file(cls, path: str) -> Self:
        with open(path, "rb") as f:
            buf = b""
            while True:
                byte = f.read(1)
                if byte == b'\0':
                    break
                buf += byte
            metadata = json.loads(buf)
            datasets = []
            while True:
                buf = f.read(8)
                if buf == b'':
                    break
                rows = int.from_bytes(buf)
                datasets.append(np.frombuffer(f.read(rows*8), float))
            return cls(metadata, datasets)


def main():
    dat = BinDat.from_file("test.dat")
    print(dat.metadata)
    for ds in dat.datasets:
        print(ds*2)
        print(len(ds))
        

if __name__ == "__main__":
    main()