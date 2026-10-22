"""Copy locally built native extensions into their development package paths."""
import argparse
from pathlib import Path
import shutil
import sys


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--release', action='store_true')
    args = parser.parse_args()
    root = Path(__file__).resolve().parent.parent
    build = root / 'target' / ('release' if args.release else 'debug')
    if sys.platform == 'darwin':
        libraries = ('lib_brimp.dylib', 'libbrimp_node.dylib')
    elif sys.platform == 'win32':
        libraries = ('_brimp.dll', 'brimp_node.dll')
    else:
        libraries = ('lib_brimp.so', 'libbrimp_node.so')
    destinations = (
        root / 'bindings/python/python/brimp' / ('_brimp.pyd' if sys.platform == 'win32' else '_brimp.abi3.so'),
        root / 'bindings/node/brimp.node',
    )
    for source, destination in zip(libraries, destinations):
        shutil.copy2(build / source, destination)
        print(destination.relative_to(root))


if __name__ == '__main__':
    main()
