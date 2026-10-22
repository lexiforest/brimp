"""Assemble relocatable macOS wheels/npm packages from already built SDK inputs.

Never modifies build outputs or SDKs. Archives have stable ordering/timestamps,
record input/output hashes, and declare the SDK's actual minimum OS version.
Other platforms retain their existing SDK packaging workflow.
"""
import argparse
import base64
import csv
import gzip
import hashlib
import io
import json
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import tarfile
import tempfile
import zipfile

ROOT = Path(__file__).resolve().parents[1]


def run(*args):
    return subprocess.check_output([str(arg) for arg in args], text=True)


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def dependencies(path):
    return [line.strip().split(' (compatibility')[0] for line in run('otool', '-L', path).splitlines()[1:]]


def minimum_os(paths):
    versions = []
    for path in paths:
        output = run('xcrun', 'vtool', '-show-build', path)
        versions.extend(tuple(map(int, version.split('.'))) for version in re.findall(r'(?:minos|version)\s+(\d+\.\d+(?:\.\d+)?)', output)[:1])
    if len(versions) != len(paths):
        raise RuntimeError('Every binary must declare its minimum macOS version')
    return max(versions)


def relocate(binary, libraries, *, extension=False):
    prefix = '@loader_path/native/' if extension else '@loader_path/'
    identifiers = run('otool', '-D', binary).splitlines()[1:]
    for dep in dependencies(binary):
        if dep in identifiers:
            continue
        name = next((name for name in libraries if ('JavaScriptCore' in dep and name == 'libJavaScriptCore.dylib') or ('libcurl-impersonate' in dep and name == 'libcurl-impersonate.dylib')), None)
        if name:
            # A dylib's own ID is handled separately below.
            if not extension and binary.name == name:
                continue
            subprocess.run(['install_name_tool', '-change', dep, prefix + name, str(binary)], check=True, capture_output=True)
        elif not dep.startswith(('/usr/lib/', '/System/Library/')):
            raise RuntimeError(f'Unbundled dependency in {binary.name}: {dep}')
    if not extension:
        subprocess.run(['install_name_tool', '-id', '@rpath/' + binary.name, str(binary)], check=True, capture_output=True)
    subprocess.run(['codesign', '--force', '--sign', '-', '--timestamp=none', str(binary)], check=True, capture_output=True)


def bundle(package, extension, libraries, notices):
    native = package / 'native'
    native.mkdir(exist_ok=True)
    manifest = {'platform': 'darwin', 'architecture': platform.machine(), 'libraries': {}}
    sources = list(libraries.values()) + [extension]
    for source in sources:
        if run("lipo", "-archs", source).split() != [platform.machine()]:
            raise RuntimeError(f"Expected a single {platform.machine()} binary: {source}")
    version = minimum_os(sources)
    manifest['minimumMacOS'] = '.'.join(map(str, version))
    for name, source in libraries.items():
        destination = native / name
        shutil.copyfile(source, destination)
        destination.chmod(0o755)
        relocate(destination, libraries)
        manifest['libraries'][name] = {'inputSha256': sha(source), 'bundledSha256': sha(destination), 'dependencies': dependencies(destination)}
    relocate(extension, libraries, extension=True)
    licenses = native / 'licenses'
    licenses.mkdir(exist_ok=True)
    copied = 0
    for index, root in enumerate(notices):
        files = [root] if root.is_file() else sorted(path for path in root.rglob('*') if path.is_file() and path.name.upper().startswith(('LICENSE', 'COPYING', 'NOTICE')))
        for source in files:
            relative = Path(source.name) if root.is_file() else source.relative_to(root)
            target = licenses / f'{index}-{root.name}' / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(source, target)
            copied += 1
    if not copied:
        raise RuntimeError('No SDK license/notice files supplied')
    (native / 'manifest.json').write_text(json.dumps(manifest, indent=2, sort_keys=True) + '\n')
    return version


def wheel_archive(root, output):
    record = next(root.glob('*.dist-info/RECORD'))
    rows = []
    for path in sorted(root.rglob('*')):
        if path.is_file() and path != record:
            data = path.read_bytes()
            digest = base64.urlsafe_b64encode(hashlib.sha256(data).digest()).rstrip(b'=').decode()
            rows.append([path.relative_to(root).as_posix(), 'sha256=' + digest, str(len(data))])
    rows.append([record.relative_to(root).as_posix(), '', ''])
    buffer = io.StringIO()
    csv.writer(buffer, lineterminator='\n').writerows(rows)
    record.write_text(buffer.getvalue())
    with zipfile.ZipFile(output, 'w', compression=zipfile.ZIP_DEFLATED) as archive:
        for path in sorted(root.rglob('*')):
            if path.is_file():
                info = zipfile.ZipInfo(path.relative_to(root).as_posix(), (1980, 1, 1, 0, 0, 0))
                info.compress_type = zipfile.ZIP_DEFLATED
                info.external_attr = (0o100755 if path.suffix in ('.so', '.dylib') else 0o100644) << 16
                archive.writestr(info, path.read_bytes())


def npm_archive(package, output):
    with output.open('wb') as stream, gzip.GzipFile(filename='', fileobj=stream, mode='wb', mtime=0) as compressed, tarfile.open(fileobj=compressed, mode='w') as archive:
        for path in sorted(package.rglob('*')):
            if path.is_file():
                info = archive.gettarinfo(str(path), 'package/' + path.relative_to(package).as_posix())
                info.uid = info.gid = info.mtime = 0
                info.uname = info.gname = ''
                with path.open('rb') as content:
                    archive.addfile(info, content)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--wheel', type=Path, required=True)
    parser.add_argument('--node', type=Path, default=ROOT / 'bindings/node')
    parser.add_argument('--jsc', type=Path, required=True)
    parser.add_argument('--curl', type=Path, required=True)
    parser.add_argument('--notices', type=Path, action='append', required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    if platform.system() != 'Darwin':
        parser.error('This relocation step supports macOS; use existing SDK builds on other platforms')
    libraries = {'libJavaScriptCore.dylib': args.jsc.resolve(strict=True), 'libcurl-impersonate.dylib': args.curl.resolve(strict=True)}
    for path in args.notices:
        path.resolve(strict=True)
    args.out.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='brimp-package-') as temporary:
        root = Path(temporary)
        wheel = root / 'wheel'
        with zipfile.ZipFile(args.wheel) as archive:
            archive.extractall(wheel)
        extension = next((wheel / 'brimp').glob('_brimp*.so'))
        version = bundle(wheel / 'brimp', extension, libraries, args.notices)
        tag = f'macosx_{version[0]}_{version[1]}_{platform.machine()}'
        name = re.sub(r'macosx_[^.]+(?=\.whl$)', tag, args.wheel.name)
        if name == args.wheel.name and 'macosx_' not in name:
            raise RuntimeError('Input wheel must be a macOS wheel')
        metadata = next(wheel.glob('*.dist-info/WHEEL'))
        metadata.write_text(re.sub(r'macosx_\S+', tag, metadata.read_text()))
        wheel_archive(wheel, args.out / name)
        node = root / 'node'
        node.mkdir()
        package = json.loads((args.node / 'package.json').read_text())
        for entry in package['files']:
            source = args.node / entry
            if source.is_dir(): shutil.copytree(source, node / entry)
            else: shutil.copyfile(source, node / entry)
        bundle(node, node / 'brimp.node', libraries, args.notices)
        package['files'] = list(dict.fromkeys(package['files'] + ['native']))
        package['os'], package['cpu'] = ['darwin'], [dict(arm64='arm64', x86_64='x64')[platform.machine()]]
        (node / 'package.json').write_text(json.dumps(package, indent=2) + '\n')
        npm_archive(node, args.out / f"{package['name']}-{package['version']}-darwin-{platform.machine()}.tgz")
    for path in sorted(args.out.iterdir()):
        if path.suffix in ('.whl', '.tgz'):
            path.with_name(path.name + '.sha256').write_text(f'{sha(path)}  {path.name}\n')
            print(path)


if __name__ == '__main__':
    main()
