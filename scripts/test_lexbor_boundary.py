"""Build and run the C boundary directly, with ASan/UBSan by default."""
import argparse
from pathlib import Path
import subprocess
import tempfile


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--no-sanitize', action='store_true')
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    lexbor = root / 'crates/lexbor'
    source = lexbor / 'vendor/lexbor/source'
    modules = ('core', 'dom', 'ns', 'tag', 'html', 'css', 'selectors', 'ports/posix')
    files = sorted(p for module in modules for p in (source / 'lexbor' / module).rglob('*.c'))
    with tempfile.TemporaryDirectory(prefix='brimp-lexbor-') as temp:
        binary = Path(temp) / 'boundary'
        command = ['clang', '-std=c11', '-D_POSIX_C_SOURCE=200809L', '-DLEXBOR_STATIC', '-O1', '-g',
                   '-I', str(source), '-I', str(lexbor / 'native')]
        if not args.no_sanitize:
            command += ['-fsanitize=address,undefined', '-fno-omit-frame-pointer']
        command += [str(p) for p in files]
        command += [str(lexbor / 'native/lexbor.c'), str(lexbor / 'native/tests/lexbor.c'), '-lm', '-o', str(binary)]
        subprocess.run(command, check=True)
        subprocess.run([str(binary)], check=True)


if __name__ == '__main__':
    main()
