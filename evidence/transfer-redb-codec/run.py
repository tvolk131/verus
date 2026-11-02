"""Entrypoint for the reusable pinned PageNumber codec replay."""
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "source/tools"))
from check_redb_codec import main

if __name__ == "__main__":
    main()
