from subprocess import check_call
import shutil
import argparse

def build_linux_en(ver: str):
    file = f'dcli_{ver}-x86_64-unknown-linux-gnu'
    print(f"building {file}")
    check_call("cargo build --release".split())
    shutil.move("./target/release/dcli", file)

def build_linux_zh(ver: str):
    file = f'dcli-zh-CN_{ver}-x86_64-unknown-linux-gnu'
    print(f"building {file}")
    check_call("cargo build --release --features zh-CN --no-default-features".split())
    shutil.move("./target/release/dcli", file)

def build_deb_en(ver: str):
    file = f'dcli_{ver}_amd64.deb'
    print(f"building {file}")
    check_call(f"cargo deb -o {file}".split())

def build_deb_zh(ver: str):
    file = f'dcli_zh_CN_{ver}_amd64.deb'
    print(f"building {file}")
    check_call(f"cargo deb -o {file} -- --features zh-CN --no-default-features".split())

def run():
    parser = argparse.ArgumentParser('dcli builder')
    parser.add_argument('ver')
    args = parser.parse_args()
    build_linux_en(args.ver)
    build_linux_zh(args.ver)
    build_deb_en(args.ver)
    build_deb_zh(args.ver)

if __name__ == "__main__":
    run()