import argparse
import pathlib
import zipfile

ROOT = pathlib.Path(__file__).resolve().parents[1]
FILES = [
    "extension/manifest.json",
    "extension/background.js",
    "extension/content.js",
    "extension/popup.html",
    "extension/popup.js",
    "extension/_locales/en/messages.json",
    "extension/_locales/pt_BR/messages.json",
    "extension/native_messaging/host_manifest.json",
    "extension/icons/icon-16.png",
    "extension/icons/icon-32.png",
    "extension/icons/icon-48.png",
    "extension/icons/icon-128.png",
]

parser = argparse.ArgumentParser(description="Create a deterministic extension ZIP")
parser.add_argument("--output", required=True)
args = parser.parse_args()
output = pathlib.Path(args.output)
output.parent.mkdir(parents=True, exist_ok=True)
with zipfile.ZipFile(output, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as archive:
    for relative in FILES:
        source = ROOT / relative
        if not source.is_file():
            raise SystemExit(f"missing extension file: {relative}")
        info = zipfile.ZipInfo(relative)
        info.date_time = (1980, 1, 1, 0, 0, 0)
        info.compress_type = zipfile.ZIP_DEFLATED
        info.external_attr = 0o644 << 16
        archive.writestr(info, source.read_bytes())
print(f"Wrote {output} with {len(FILES)} deterministic entries")
